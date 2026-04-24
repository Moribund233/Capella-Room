"""
压力测试脚本 - 验证服务在5GB内存限制下的性能上限

测试场景：
1. 逐步增加并发用户数（50, 100, 200, 300, 500）
2. 每个用户发送多条消息
3. 监控内存使用、响应时间、错误率
4. 找出系统的性能拐点
"""

import asyncio
import aiohttp
import websockets
import json
import time
import sys
from dataclasses import dataclass, field
from typing import List, Dict, Optional
from concurrent.futures import ThreadPoolExecutor
import statistics
from dotenv import load_dotenv
import os

# 加载环境变量
load_dotenv('../.env')

BASE_URL = os.getenv('TEST_BASE_URL', 'http://localhost:8765')
WS_URL = os.getenv('TEST_WS_URL', 'ws://localhost:8765/ws')


@dataclass
class StressTestResult:
    """压力测试结果"""
    concurrent_users: int
    total_messages: int
    success_count: int
    error_count: int
    avg_response_time: float
    max_response_time: float
    min_response_time: float
    memory_before: float
    memory_after: float
    errors: List[str] = field(default_factory=list)


@dataclass
class UserMetrics:
    """用户性能指标"""
    user_id: str
    connect_time: float = 0
    messages_sent: int = 0
    messages_received: int = 0
    errors: List[str] = field(default_factory=list)


class StressTestClient:
    """压力测试客户端"""
    
    def __init__(self):
        self.base_url = BASE_URL
        self.ws_url = WS_URL
        self.session: Optional[aiohttp.ClientSession] = None
        
    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()
    
    async def login(self, email: str, password: str) -> Optional[str]:
        """用户登录获取token"""
        try:
            async with self.session.post(
                f"{self.base_url}/api/auth/login",
                json={"email": email, "password": password}
            ) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    return data.get('data', {}).get('access_token')
                return None
        except Exception as e:
            return None
    
    async def connect_websocket(self, token: str, user_email: str = "") -> Optional[websockets.WebSocketClientProtocol]:
        """建立WebSocket连接并认证"""
        try:
            ws = await websockets.connect(
                self.ws_url,
                open_timeout=10,
                close_timeout=5
            )
            
            # 发送认证消息
            auth_msg = {
                "type": "Auth",
                "payload": {"token": token}
            }
            await ws.send(json.dumps(auth_msg))
            
            # 等待认证结果
            response = await asyncio.wait_for(ws.recv(), timeout=5.0)
            resp_data = json.loads(response)
            
            if resp_data.get('type') == 'AuthResult' and resp_data.get('payload', {}).get('success'):
                return ws
            else:
                print(f"[WebSocket Auth Failed] {user_email}: {resp_data}")
                await ws.close()
                return None
                
        except Exception as e:
            print(f"[WebSocket Error] {user_email}: {type(e).__name__}: {str(e)[:100]}")
            return None
    
    async def user_session_worker(
        self,
        user_email: str,
        password: str,
        room_id: str,
        messages_per_user: int,
        metrics: UserMetrics
    ):
        """单个用户会话工作线程"""
        start_time = time.time()
        ws = None
        
        try:
            # 登录
            token = await self.login(user_email, password)
            if not token:
                metrics.errors.append(f"登录失败: {user_email}")
                return
            
            # 连接WebSocket
            ws = await self.connect_websocket(token, user_email)
            if not ws:
                metrics.errors.append(f"WebSocket连接失败: {user_email}")
                return
            
            metrics.connect_time = time.time() - start_time
            
            # 加入房间
            join_msg = {
                "type": "JoinRoom",
                "payload": {"room_id": room_id}
            }
            await ws.send(json.dumps(join_msg))
            
            # 等待加入确认（可能需要跳过Ping消息）
            joined = False
            for _ in range(10):  # 最多尝试10次
                try:
                    response = await asyncio.wait_for(ws.recv(), timeout=2.0)
                    resp_data = json.loads(response)
                    msg_type = resp_data.get('type')
                    
                    if msg_type == 'RoomJoined':
                        joined = True
                        break
                    elif msg_type == 'Ping':
                        # 回复Pong
                        await ws.send(json.dumps({"type": "Pong"}))
                        continue
                    elif msg_type == 'Error':
                        metrics.errors.append(f"加入房间错误: {user_email}, {resp_data}")
                        return
                        
                except asyncio.TimeoutError:
                    break
            
            if not joined:
                metrics.errors.append(f"加入房间失败或超时: {user_email}")
                return
            
            # 发送消息
            for i in range(messages_per_user):
                msg = {
                    "type": "SendMessage",
                    "payload": {
                        "room_id": room_id,
                        "content": f"压力测试消息-{user_email}-{i}-{int(time.time())}"
                    }
                }
                try:
                    await ws.send(json.dumps(msg))
                    metrics.messages_sent += 1
                    
                    # 接收消息（包括自己发送的广播）
                    try:
                        response = await asyncio.wait_for(ws.recv(), timeout=2.0)
                        metrics.messages_received += 1
                    except asyncio.TimeoutError:
                        pass  # 消息可能还在路上
                        
                except Exception as e:
                    metrics.errors.append(f"发送消息失败: {str(e)}")
            
            # 等待接收其他用户的消息
            await asyncio.sleep(2)
            
            # 尝试接收剩余消息
            while True:
                try:
                    response = await asyncio.wait_for(ws.recv(), timeout=0.5)
                    metrics.messages_received += 1
                except asyncio.TimeoutError:
                    break
            
        except Exception as e:
            metrics.errors.append(f"会话错误: {str(e)}")
        finally:
            if ws:
                await ws.close()


class StressTester:
    """压力测试执行器"""
    
    def __init__(self):
        self.results: List[StressTestResult] = []
        self.test_users = [
            {'email': f'TestUser{i}@test.com', 'password': 'Test12345'}
            for i in range(3, 103)  # TestUser3-102, 共100个用户
        ]
    
    def log(self, message: str):
        """打印日志"""
        timestamp = time.strftime('%Y-%m-%d %H:%M:%S')
        print(f"[{timestamp}] {message}")
    
    async def get_container_memory(self) -> float:
        """获取容器内存占用（MB）"""
        import subprocess
        try:
            result = subprocess.run(
                ['wsl', 'docker', 'stats', '--no-stream', 'seredeli-room-prod'],
                capture_output=True,
                text=True
            )
            lines = result.stdout.strip().split('\n')
            if len(lines) >= 2:
                parts = lines[1].split()
                for i, part in enumerate(parts):
                    if 'MiB' in part:
                        return float(part.replace('MiB', ''))
                    elif 'GiB' in part:
                        return float(part.replace('GiB', '')) * 1024
        except Exception as e:
            self.log(f"获取内存失败: {e}")
        return 0.0
    
    async def create_room_for_test(
        self, 
        client: StressTestClient, 
        room_name: str, 
        max_members: int
    ) -> Optional[str]:
        """为测试创建房间"""
        token = await client.login(self.test_users[0]['email'], self.test_users[0]['password'])
        if not token:
            return None
        
        try:
            async with client.session.post(
                f"{BASE_URL}/api/rooms",
                headers={"Authorization": f"Bearer {token}"},
                json={
                    "name": room_name,
                    "description": f"压力测试房间",
                    "is_private": False,
                    "max_members": max_members
                }
            ) as resp:
                if resp.status in [200, 201]:
                    data = await resp.json()
                    if data.get('success'):
                        return data.get('data', {}).get('id')
        except Exception as e:
            self.log(f"创建房间异常: {e}")
        return None
    
    async def run_stress_test_multi_room(
        self,
        concurrent_users: int,
        messages_per_user: int,
        users_per_room: int = 100
    ) -> StressTestResult:
        """执行多房间分片压力测试"""
        self.log(f"\n{'='*60}")
        self.log(f"压力测试: {concurrent_users} 并发用户 (多房间分片)")
        self.log(f"每个用户发送: {messages_per_user} 条消息")
        self.log(f"每房间用户数: {users_per_room}")
        self.log(f"总消息数: {concurrent_users * messages_per_user}")
        self.log('='*60)
        
        # 记录测试前内存
        memory_before = await self.get_container_memory()
        self.log(f"测试前内存: {memory_before:.1f} MiB")
        
        # 计算需要的房间数
        num_rooms = (concurrent_users + users_per_room - 1) // users_per_room
        self.log(f"需要创建房间数: {num_rooms}")
        
        # 创建多个房间
        room_ids = []
        async with StressTestClient() as client:
            for i in range(num_rooms):
                room_id = await self.create_room_for_test(
                    client, 
                    f"压力测试房间-{int(time.time())}-{i}",
                    users_per_room + 10
                )
                if room_id:
                    room_ids.append(room_id)
                    self.log(f"  房间 {i+1} 创建成功: {room_id}")
                else:
                    self.log(f"  房间 {i+1} 创建失败")
        
        if not room_ids:
            return StressTestResult(
                concurrent_users=concurrent_users,
                total_messages=0,
                success_count=0,
                error_count=concurrent_users,
                avg_response_time=0,
                max_response_time=0,
                min_response_time=0,
                memory_before=memory_before,
                memory_after=memory_before,
                errors=["房间创建失败"]
            )
        
        # 为每个用户分配房间（循环复用测试账号）
        user_room_assignments = []
        user_index_map = []  # 记录每个并发用户对应的测试用户索引
        for i in range(concurrent_users):
            room_idx = i // users_per_room
            if room_idx < len(room_ids):
                user_room_assignments.append(room_ids[room_idx])
            else:
                user_room_assignments.append(room_ids[-1])
            # 循环复用测试用户
            user_index_map.append(i % len(self.test_users))
        
        # 准备用户指标
        user_metrics: List[UserMetrics] = []
        for i in range(concurrent_users):
            user_idx = user_index_map[i]
            user_metrics.append(UserMetrics(user_id=f"{self.test_users[user_idx]['email']}#{i}"))
        
        # 执行并发测试
        start_time = time.time()
        
        async with StressTestClient() as client:
            tasks = []
            for i in range(concurrent_users):
                user_idx = user_index_map[i]
                task = client.user_session_worker(
                    self.test_users[user_idx]['email'],
                    self.test_users[user_idx]['password'],
                    user_room_assignments[i],
                    messages_per_user,
                    user_metrics[i]
                )
                tasks.append(task)
            
            # 使用信号量限制并发连接数
            semaphore = asyncio.Semaphore(30)  # 最多30个并发连接
            
            async def limited_task(task):
                async with semaphore:
                    await task
                    await asyncio.sleep(0.05)  # 连接间隔50ms
            
            limited_tasks = [limited_task(t) for t in tasks]
            await asyncio.gather(*limited_tasks, return_exceptions=True)
        
        total_time = time.time() - start_time
        
        # 记录测试后内存
        await asyncio.sleep(2)
        memory_after = await self.get_container_memory()
        self.log(f"测试后内存: {memory_after:.1f} MiB")
        
        # 统计结果
        total_sent = sum(m.messages_sent for m in user_metrics)
        total_received = sum(m.messages_received for m in user_metrics)
        connect_times = [m.connect_time for m in user_metrics if m.connect_time > 0]
        all_errors = []
        for m in user_metrics:
            all_errors.extend(m.errors)
        
        success_count = sum(1 for m in user_metrics if m.messages_sent > 0)
        error_count = concurrent_users - success_count
        
        # 计算响应时间统计
        if connect_times:
            avg_time = statistics.mean(connect_times)
            max_time = max(connect_times)
            min_time = min(connect_times)
        else:
            avg_time = max_time = min_time = 0
        
        result = StressTestResult(
            concurrent_users=concurrent_users,
            total_messages=concurrent_users * messages_per_user,
            success_count=success_count,
            error_count=error_count,
            avg_response_time=avg_time,
            max_response_time=max_time,
            min_response_time=min_time,
            memory_before=memory_before,
            memory_after=memory_after,
            errors=all_errors[:10]
        )
        
        # 输出结果
        self.log(f"\n测试结果:")
        self.log(f"  总耗时: {total_time:.2f} 秒")
        self.log(f"  成功用户数: {success_count}/{concurrent_users}")
        self.log(f"  消息发送: {total_sent}/{concurrent_users * messages_per_user}")
        self.log(f"  消息接收: {total_received}")
        self.log(f"  平均连接时间: {avg_time:.3f} 秒")
        self.log(f"  最大连接时间: {max_time:.3f} 秒")
        self.log(f"  内存增长: {memory_after - memory_before:.1f} MiB")
        self.log(f"  错误数: {len(all_errors)}")
        
        if all_errors:
            self.log(f"  错误示例: {all_errors[:3]}")
        
        return result
    
    async def run_full_stress_test(self):
        """执行完整压力测试套件"""
        self.log("\n" + "="*60)
        self.log("开始压力测试套件")
        self.log("容器内存限制: 5GB")
        self.log("="*60)
        
        # 测试配置: (并发用户数, 每用户消息数, 每房间用户数)
        # 注意：我们只有100个测试用户(TestUser3-102)，通过循环复用实现更高并发
        test_scenarios = [
            (100, 5, 100),   # 100用户，每人5条，1个房间
            (200, 3, 100),   # 200用户（复用2次），每人3条，2个房间
            (500, 2, 100),   # 500用户（复用5次），每人2条，5个房间
        ]
        
        # 执行测试
        for users, msgs, room_size in test_scenarios:
            result = await self.run_stress_test_multi_room(users, msgs, room_size)
            self.results.append(result)
            
            # 如果成功率低于90%，停止测试
            success_rate = result.success_count / result.concurrent_users
            if success_rate < 0.9:
                self.log(f"\n⚠️ 成功率 {success_rate*100:.1f}% 低于90%，停止增加负载")
                break
            
            # 如果内存超过4GB，停止测试
            if result.memory_after > 4096:
                self.log(f"\n⚠️ 内存使用 {result.memory_after:.0f}MiB 接近限制，停止增加负载")
                break
            
            # 测试间隔
            if (users, msgs, room_size) != test_scenarios[-1]:
                self.log(f"\n等待10秒进行下一次测试...")
                await asyncio.sleep(10)
        
        # 输出最终报告
        self.print_final_report()
    
    def print_final_report(self):
        """打印最终测试报告"""
        self.log("\n" + "="*60)
        self.log("压力测试最终报告")
        self.log("="*60)
        
        if not self.results:
            self.log("没有测试结果")
            return
        
        self.log(f"\n{'并发用户':<10} {'成功率':<10} {'平均响应':<12} {'内存增长':<12} {'状态'}")
        self.log("-" * 60)
        
        max_successful_users = 0
        for r in self.results:
            success_rate = (r.success_count / r.concurrent_users) * 100
            status = "✅ 通过" if success_rate >= 90 else "❌ 失败"
            memory_growth = r.memory_after - r.memory_before
            
            self.log(f"{r.concurrent_users:<10} {success_rate:>6.1f}%    {r.avg_response_time:>8.3f}s    {memory_growth:>8.1f}MiB   {status}")
            
            if success_rate >= 90:
                max_successful_users = r.concurrent_users
        
        self.log("\n" + "="*60)
        self.log(f"性能上限估算: 约 {max_successful_users} 并发用户")
        
        # 基于5GB内存的容量规划
        if self.results:
            # 找到最后一个成功的测试
            successful_results = [r for r in self.results if r.success_count > 0]
            if successful_results:
                last_result = successful_results[-1]
                memory_per_user = (last_result.memory_after - last_result.memory_before) / last_result.success_count
                estimated_max_users = int(5120 / memory_per_user) if memory_per_user > 0 else 0  # 5GB = 5120MiB
                
                self.log(f"单用户内存占用: {memory_per_user:.1f} MiB")
                self.log(f"5GB内存理论容量: 约 {estimated_max_users} 并发用户")
            else:
                self.log("没有成功的测试，无法进行容量规划")
        
        self.log("="*60)


async def main():
    """主函数"""
    tester = StressTester()
    await tester.run_full_stress_test()


if __name__ == "__main__":
    asyncio.run(main())
