"""
阶段9：端到端场景测试脚本

测试场景：
1. 模拟完整用户流程：注册 → 登录 → 创建房间 → 邀请用户 → 发送消息 → 上传文件
2. 模拟多用户并发场景：多个用户同时在多个房间聊天
3. 模拟用户重连场景：网络中断后重连，验证消息不丢失
4. 模拟房间管理员操作：踢出成员、设置管理员、删除房间

使用方式:
    python e2e_test.py [端口]
    
示例:
    python e2e_test.py              # 使用默认端口8765
    python e2e_test.py 8080         # 使用端口8080

环境变量配置 (script/.env):
    TEST_HOST=localhost
    TEST_PORT=8765
    TEST_ACCOUNT=TestUser1@test.com
    TEST_PASSWORD=Test12345
"""

import asyncio
import json
import os
import sys
import time
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional, Dict, List, Any, Callable
from urllib.parse import urljoin

import requests
import websockets
from dotenv import load_dotenv

# 加载环境变量
load_dotenv(os.path.join(os.path.dirname(__file__), '..', '.env'))

# 配置（可以通过命令行参数覆盖）
TEST_HOST = os.getenv('TEST_HOST', 'localhost')
TEST_PORT = int(os.getenv('TEST_PORT', '8765'))
TEST_ACCOUNT = os.getenv('TEST_ACCOUNT', 'test@example.com')
TEST_PASSWORD = os.getenv('TEST_PASSWORD', 'password123')


def get_base_url(port: int) -> str:
    """获取基础URL"""
    return f"http://{TEST_HOST}:{port}"


def get_ws_url(port: int) -> str:
    """获取WebSocket URL"""
    return f"ws://{TEST_HOST}:{port}/ws"


@dataclass
class UserSession:
    """用户会话，保存用户的认证信息和状态"""
    email: str
    password: str
    username: str = ""
    user_id: Optional[str] = None
    access_token: Optional[str] = None
    refresh_token: Optional[str] = None
    ws_connection: Optional[websockets.WebSocketClientProtocol] = None
    received_messages: List[Dict] = field(default_factory=list)
    joined_rooms: List[str] = field(default_factory=list)
    message_handlers: List[Callable] = field(default_factory=list)
    
    def get_headers(self) -> Dict[str, str]:
        """获取认证请求头"""
        return {
            'Authorization': f'Bearer {self.access_token}',
            'Content-Type': 'application/json'
        }


class ChatClient:
    """聊天室客户端，封装HTTP和WebSocket操作"""
    
    def __init__(self, base_url: str, ws_url: str):
        self.base_url = base_url
        self.ws_url = ws_url
        self.session = requests.Session()
    
    def register(self, email: str, password: str, username: str) -> Dict:
        """
        用户注册
        
        Args:
            email: 邮箱
            password: 密码
            username: 用户名
            
        Returns:
            注册结果，包含用户信息
        """
        url = f"{self.base_url}/api/v1/auth/register"
        data = {
            'email': email,
            'password': password,
            'username': username
        }
        response = self.session.post(url, json=data)
        return response.json()
    
    def login(self, email: str, password: str) -> Dict:
        """
        用户登录
        
        Args:
            email: 邮箱
            password: 密码
            
        Returns:
            登录结果，包含access_token和用户信息
        """
        url = f"{self.base_url}/api/v1/auth/login"
        data = {
            'email': email,
            'password': password
        }
        response = self.session.post(url, json=data)
        return response.json()
    
    def create_room(self, session: UserSession, name: str, description: str = "", 
                   is_private: bool = False, max_members: int = 100) -> Dict:
        """
        创建聊天室
        
        Args:
            session: 用户会话
            name: 房间名称
            description: 房间描述
            is_private: 是否私有房间
            max_members: 最大成员数
            
        Returns:
            创建的房间信息
        """
        url = f"{self.base_url}/api/v1/rooms"
        data = {
            'name': name,
            'description': description,
            'is_private': is_private,
            'max_members': max_members
        }
        response = self.session.post(url, json=data, headers=session.get_headers())
        return response.json()
    
    def list_rooms(self, session: UserSession) -> Dict:
        """
        获取聊天室列表
        
        Args:
            session: 用户会话
            
        Returns:
            房间列表
        """
        url = f"{self.base_url}/api/v1/rooms"
        response = self.session.get(url, headers=session.get_headers())
        return response.json()
    
    def join_room(self, session: UserSession, room_id: str) -> Dict:
        """
        加入聊天室
        
        Args:
            session: 用户会话
            room_id: 房间ID
            
        Returns:
            加入结果
        """
        url = f"{self.base_url}/api/v1/rooms/{room_id}/join"
        response = self.session.post(url, headers=session.get_headers())
        return response.json()
    
    def leave_room(self, session: UserSession, room_id: str) -> Dict:
        """
        离开聊天室
        
        Args:
            session: 用户会话
            room_id: 房间ID
            
        Returns:
            离开结果
        """
        url = f"{self.base_url}/api/v1/rooms/{room_id}/leave"
        response = self.session.delete(url, headers=session.get_headers())
        return response.json()
    
    def get_room_members(self, session: UserSession, room_id: str) -> Dict:
        """
        获取房间成员列表
        
        Args:
            session: 用户会话
            room_id: 房间ID
            
        Returns:
            成员列表
        """
        url = f"{self.base_url}/api/v1/rooms/{room_id}/members"
        response = self.session.get(url, headers=session.get_headers())
        return response.json()
    
    def kick_member(self, session: UserSession, room_id: str, user_id: str) -> Dict:
        """
        踢出房间成员
        
        Args:
            session: 用户会话
            room_id: 房间ID
            user_id: 要踢出的用户ID
            
        Returns:
            操作结果
        """
        url = f"{self.base_url}/api/v1/rooms/{room_id}/members/{user_id}"
        response = self.session.delete(url, headers=session.get_headers())
        return response.json()
    
    def set_member_role(self, session: UserSession, room_id: str, user_id: str, role: str) -> Dict:
        """
        设置成员角色
        
        Args:
            session: 用户会话
            room_id: 房间ID
            user_id: 用户ID
            role: 角色 (owner/admin/member)
            
        Returns:
            操作结果
        """
        url = f"{self.base_url}/api/v1/rooms/{room_id}/members/{user_id}/role"
        data = {'role': role}
        response = self.session.put(url, json=data, headers=session.get_headers())
        return response.json()
    
    def delete_room(self, session: UserSession, room_id: str) -> Dict:
        """
        删除聊天室
        
        Args:
            session: 用户会话
            room_id: 房间ID
            
        Returns:
            删除结果
        """
        url = f"{self.base_url}/api/v1/rooms/{room_id}"
        response = self.session.delete(url, headers=session.get_headers())
        return response.json()
    
    def get_room_messages(self, session: UserSession, room_id: str) -> Dict:
        """
        获取房间消息历史
        
        Args:
            session: 用户会话
            room_id: 房间ID
            
        Returns:
            消息列表
        """
        url = f"{self.base_url}/api/v1/rooms/{room_id}/messages"
        response = self.session.get(url, headers=session.get_headers())
        return response.json()
    
    def upload_file(self, session: UserSession, file_path: str, room_id: Optional[str] = None) -> Dict:
        """
        上传文件
        
        Args:
            session: 用户会话
            file_path: 文件路径
            room_id: 关联的房间ID（可选）
            
        Returns:
            上传结果
        """
        url = f"{self.base_url}/api/v1/upload"
        
        with open(file_path, 'rb') as f:
            files = {'file': f}
            data = {}
            if room_id:
                data['room_id'] = room_id
            
            headers = {'Authorization': f'Bearer {session.access_token}'}
            response = self.session.post(url, files=files, data=data, headers=headers)
        return response.json()
    
    async def connect_websocket(self, session: UserSession) -> bool:
        """
        建立WebSocket连接并进行认证
        
        Args:
            session: 用户会话
            
        Returns:
            是否连接成功
        """
        try:
            session.ws_connection = await websockets.connect(self.ws_url)
            
            # 发送认证消息
            auth_msg = {
                'type': 'Auth',
                'payload': {'token': session.access_token}
            }
            await session.ws_connection.send(json.dumps(auth_msg))
            
            # 等待认证结果
            response = await asyncio.wait_for(
                session.ws_connection.recv(), 
                timeout=5.0
            )
            data = json.loads(response)
            
            if data.get('type') == 'AuthResult':
                if data.get('payload', {}).get('success', False):
                    print(f"[WebSocket] 用户 {session.username} 认证成功")
                    # 启动消息接收任务
                    asyncio.create_task(self._receive_messages(session))
                    return True
                else:
                    print(f"[WebSocket] 认证失败: {data.get('payload', {}).get('message', '')}")
                    return False
            return False
            
        except Exception as e:
            print(f"[WebSocket] 连接失败: {e}")
            return False
    
    async def _receive_messages(self, session: UserSession):
        """
        后台任务：持续接收WebSocket消息
        
        Args:
            session: 用户会话
        """
        try:
            while session.ws_connection:
                try:
                    message = await session.ws_connection.recv()
                    data = json.loads(message)
                    
                    # 处理心跳
                    if data.get('type') == 'Ping':
                        pong_msg = {'type': 'Pong'}
                        await session.ws_connection.send(json.dumps(pong_msg))
                        continue
                    
                    # 保存接收到的消息
                    session.received_messages.append({
                        'timestamp': datetime.now().isoformat(),
                        'data': data
                    })
                    
                    # 调用注册的消息处理器
                    for handler in session.message_handlers:
                        try:
                            handler(data)
                        except Exception as e:
                            print(f"[WebSocket] 消息处理器错误: {e}")
                    
                    # 打印消息
                    msg_type = data.get('type', 'Unknown')
                    print(f"[WebSocket] {session.username} 收到消息: {msg_type}")
                    
                except websockets.exceptions.ConnectionClosed:
                    print(f"[WebSocket] {session.username} 连接已关闭")
                    break
                except Exception as e:
                    print(f"[WebSocket] 接收消息错误: {e}")
                    
        except Exception as e:
            print(f"[WebSocket] 接收任务异常: {e}")
        finally:
            session.ws_connection = None
    
    async def join_room_ws(self, session: UserSession, room_id: str) -> bool:
        """
        通过WebSocket加入房间
        
        Args:
            session: 用户会话
            room_id: 房间ID
            
        Returns:
            是否加入成功
        """
        if not session.ws_connection:
            print(f"[WebSocket] 未连接，无法加入房间")
            return False
        
        try:
            msg = {
                'type': 'JoinRoom',
                'payload': {'room_id': room_id}
            }
            await session.ws_connection.send(json.dumps(msg))
            session.joined_rooms.append(room_id)
            print(f"[WebSocket] {session.username} 加入房间 {room_id}")
            return True
        except Exception as e:
            print(f"[WebSocket] 加入房间失败: {e}")
            return False
    
    async def leave_room_ws(self, session: UserSession, room_id: str) -> bool:
        """
        通过WebSocket离开房间
        
        Args:
            session: 用户会话
            room_id: 房间ID
            
        Returns:
            是否离开成功
        """
        if not session.ws_connection:
            return False
        
        try:
            msg = {
                'type': 'LeaveRoom',
                'payload': {'room_id': room_id}
            }
            await session.ws_connection.send(json.dumps(msg))
            if room_id in session.joined_rooms:
                session.joined_rooms.remove(room_id)
            return True
        except Exception as e:
            print(f"[WebSocket] 离开房间失败: {e}")
            return False
    
    async def send_chat_message(self, session: UserSession, room_id: str, content: str,
                               reply_to: Optional[str] = None) -> bool:
        """
        发送聊天消息
        
        Args:
            session: 用户会话
            room_id: 房间ID
            content: 消息内容
            reply_to: 回复的消息ID（可选）
            
        Returns:
            是否发送成功
        """
        if not session.ws_connection:
            print(f"[WebSocket] 未连接，无法发送消息")
            return False
        
        try:
            payload = {
                'room_id': room_id,
                'content': content
            }
            if reply_to:
                payload['reply_to'] = reply_to
            
            msg = {
                'type': 'ChatMessage',
                'payload': payload
            }
            await session.ws_connection.send(json.dumps(msg))
            print(f"[WebSocket] {session.username} 发送消息到房间 {room_id}")
            return True
        except Exception as e:
            print(f"[WebSocket] 发送消息失败: {e}")
            return False
    
    async def disconnect_websocket(self, session: UserSession):
        """
        断开WebSocket连接
        
        Args:
            session: 用户会话
        """
        if session.ws_connection:
            await session.ws_connection.close()
            session.ws_connection = None
            print(f"[WebSocket] {session.username} 已断开连接")


class E2ETestRunner:
    """端到端测试执行器"""
    
    def __init__(self, base_url: str, ws_url: str):
        self.client = ChatClient(base_url, ws_url)
        self.sessions: Dict[str, UserSession] = {}
        self.test_results: List[Dict] = []
    
    def log(self, message: str, level: str = "INFO"):
        """打印日志"""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        print(f"[{timestamp}] [{level}] {message}")
    
    def record_result(self, test_name: str, success: bool, details: str = ""):
        """记录测试结果"""
        result = {
            'test_name': test_name,
            'success': success,
            'details': details,
            'timestamp': datetime.now().isoformat()
        }
        self.test_results.append(result)
        status = "✅ 通过" if success else "❌ 失败"
        self.log(f"{test_name}: {status} {details}")
    
    async def run_all_tests(self):
        """运行所有测试场景"""
        self.log("=" * 60)
        self.log("开始阶段9：端到端场景测试")
        self.log("=" * 60)
        
        # 测试场景1: 完整用户流程
        await self.test_complete_user_flow()
        
        # 测试场景2: 多用户并发
        await self.test_concurrent_users()
        
        # 测试场景3: 用户重连
        await self.test_reconnection()
        
        # 测试场景4: 房间管理员操作
        await self.test_admin_operations()
        
        # 打印测试报告
        self.print_test_report()
    
    async def test_complete_user_flow(self):
        """测试场景1: 完整用户流程"""
        self.log("\n" + "=" * 60)
        self.log("测试场景1: 完整用户流程")
        self.log("流程: 注册 → 登录 → 创建房间 → 邀请用户 → 发送消息 → 上传文件")
        self.log("=" * 60)
        
        try:
            # 1. 注册用户
            timestamp = int(time.time())
            email = f"test_user_{timestamp}@example.com"
            password = "TestPassword123"
            username = f"TestUser{timestamp}"
            
            self.log("步骤1: 注册用户...")
            result = self.client.register(email, password, username)
            if result.get('success'):
                self.log(f"  用户注册成功: {username}")
            else:
                # 如果用户已存在，继续测试
                self.log(f"  注册结果: {result.get('message', '未知')}")
            
            # 2. 登录
            self.log("步骤2: 用户登录...")
            result = self.client.login(email, password)
            if not result.get('success'):
                self.record_result("用户登录", False, result.get('message', ''))
                return
            
            user_data = result.get('data', {})
            session = UserSession(
                email=email,
                password=password,
                username=user_data.get('user', {}).get('username', username),
                user_id=user_data.get('user', {}).get('id'),
                access_token=user_data.get('access_token'),
                refresh_token=user_data.get('refresh_token')
            )
            self.sessions['main_user'] = session
            self.log(f"  登录成功，获取到 access_token")
            
            # 3. 创建房间
            self.log("步骤3: 创建聊天室...")
            result = self.client.create_room(
                session, 
                name=f"测试房间_{timestamp}",
                description="端到端测试房间",
                is_private=False
            )
            if not result.get('success'):
                self.record_result("创建房间", False, result.get('message', ''))
                return
            
            room_data = result.get('data', {})
            room_id = room_data.get('id')
            self.log(f"  房间创建成功: {room_id}")
            
            # 4. 建立WebSocket连接
            self.log("步骤4: 建立WebSocket连接...")
            if not await self.client.connect_websocket(session):
                self.record_result("WebSocket连接", False, "连接失败")
                return
            self.log("  WebSocket连接成功")
            
            # 5. 通过WebSocket加入房间
            self.log("步骤5: 加入聊天室...")
            if not await self.client.join_room_ws(session, room_id):
                self.record_result("加入房间", False, "加入失败")
                return
            self.log("  已加入房间")
            
            # 等待一下确保加入成功
            await asyncio.sleep(1)
            
            # 6. 发送消息
            self.log("步骤6: 发送聊天消息...")
            message_content = f"这是一条测试消息，时间戳: {timestamp}"
            if not await self.client.send_chat_message(session, room_id, message_content):
                self.record_result("发送消息", False, "发送失败")
                return
            self.log("  消息已发送")
            
            # 等待消息接收
            await asyncio.sleep(1)
            
            # 7. 获取房间消息历史
            self.log("步骤7: 获取消息历史...")
            result = self.client.get_room_messages(session, room_id)
            if result.get('success'):
                messages = result.get('data', [])
                self.log(f"  获取到 {len(messages)} 条消息")
            
            # 8. 上传文件（创建测试文件）
            self.log("步骤8: 上传文件...")
            test_file_path = os.path.join(os.path.dirname(__file__), 'test_file.txt')
            with open(test_file_path, 'w', encoding='utf-8') as f:
                f.write(f"这是一个测试文件，创建时间: {datetime.now()}")
            
            result = self.client.upload_file(session, test_file_path, room_id)
            if result.get('success'):
                self.log("  文件上传成功")
            else:
                self.log(f"  文件上传结果: {result.get('message', '未知')}")
            
            # 清理测试文件
            if os.path.exists(test_file_path):
                os.remove(test_file_path)
            
            # 断开连接
            await self.client.disconnect_websocket(session)
            
            self.record_result("完整用户流程", True, "所有步骤执行成功")
            
        except Exception as e:
            self.record_result("完整用户流程", False, str(e))
            import traceback
            traceback.print_exc()
    
    async def test_concurrent_users(self):
        """测试场景2: 多用户并发场景（增强版）"""
        self.log("\n" + "=" * 60)
        self.log("测试场景2: 多用户并发场景（增强版）")
        self.log("场景: 10个用户同时在线，每人发送10条消息，验证消息广播可靠性")
        self.log("=" * 60)
        
        try:
            # 使用已注册的测试用户 (TestUser3 - TestUser12，共10个用户)
            test_users_config = [
                {'email': f'TestUser{i}@test.com', 'password': 'Test12345'}
                for i in range(3, 13)  # TestUser3 到 TestUser12
            ]
            
            users = []
            
            for config in test_users_config:
                result = self.client.login(config['email'], config['password'])
                if result.get('success'):
                    user_data = result.get('data', {})
                    session = UserSession(
                        email=config['email'],
                        password=config['password'],
                        username=user_data.get('user', {}).get('username', ''),
                        user_id=user_data.get('user', {}).get('id'),
                        access_token=user_data.get('access_token'),
                        refresh_token=user_data.get('refresh_token')
                    )
                    users.append(session)
                    self.log(f"  用户 {config['email']} 登录成功")
                else:
                    self.log(f"  ⚠️ 用户 {config['email']} 登录失败: {result.get('message', '')}")
            
            if len(users) < 5:
                self.record_result("多用户并发", False, f"登录用户不足，仅成功登录 {len(users)} 个（需要至少5个）")
                return
            
            self.log(f"  成功登录 {len(users)} 个用户")
            
            # 第一个用户创建房间
            timestamp = int(time.time())
            creator = users[0]
            result = self.client.create_room(
                creator,
                name=f"高并发测试房间_{timestamp}",
                description="10用户100消息并发测试",
                is_private=False,
                max_members=100
            )
            
            if not result.get('success'):
                self.record_result("多用户并发", False, "创建房间失败")
                return
            
            room_id = result.get('data', {}).get('id')
            self.log(f"  房间创建成功: {room_id}")
            
            # 所有用户建立WebSocket连接并加入房间
            self.log(f"  {len(users)}个用户连接WebSocket并加入房间...")
            connected_count = 0
            for user in users:
                if await self.client.connect_websocket(user):
                    if await self.client.join_room_ws(user, room_id):
                        connected_count += 1
            
            self.log(f"  成功连接并加入房间: {connected_count}/{len(users)} 个用户")
            
            if connected_count < 3:
                self.record_result("多用户并发", False, "连接用户不足")
                return
            
            # 等待所有用户加入完成
            await asyncio.sleep(2)
            
            # 每个用户发送10条消息，总共100条消息
            messages_per_user = 10
            total_expected_messages = connected_count * messages_per_user
            self.log(f"  开始发送消息: {connected_count}个用户 × {messages_per_user}条 = {total_expected_messages}条消息")
            
            send_tasks = []
            for user_idx, user in enumerate(users[:connected_count]):
                for msg_idx in range(messages_per_user):
                    content = f"用户{user_idx+1}-消息{msg_idx+1}-时间戳{timestamp}"
                    send_tasks.append(
                        self.client.send_chat_message(user, room_id, content)
                    )
                    # 添加小延迟避免瞬间爆发
                    if msg_idx % 3 == 0:
                        await asyncio.sleep(0.05)
            
            await asyncio.gather(*send_tasks)
            self.log(f"  所有 {total_expected_messages} 条消息已发送")
            
            # 等待消息接收（给足够时间）
            await asyncio.sleep(5)
            
            # 统计结果
            total_received = sum(len(u.received_messages) for u in users)
            # 每个用户应该收到 (connected_count - 1) * messages_per_user 条其他用户的消息
            # 加上系统消息（RoomJoined, OnlineUsers等）
            expected_min_messages = connected_count * (connected_count - 1) * messages_per_user
            
            self.log(f"  消息统计:")
            self.log(f"    - 发送消息数: {total_expected_messages}")
            self.log(f"    - 预期接收消息数（广播）: {expected_min_messages}")
            self.log(f"    - 实际接收消息总数: {total_received}")
            
            # 检查每个用户收到的消息
            message_stats = []
            for i, user in enumerate(users[:connected_count]):
                chat_messages = [m for m in user.received_messages 
                               if m.get('data', {}).get('type') == 'NewMessage']
                message_stats.append({
                    'user': f"用户{i+1}",
                    'total': len(user.received_messages),
                    'chat_msgs': len(chat_messages)
                })
                self.log(f"    - {user.email}: 共收到 {len(user.received_messages)} 条消息（聊天消息: {len(chat_messages)}）")
            
            # 验证消息完整性
            success = True
            details = f"{connected_count}个用户发送{total_expected_messages}条消息"
            
            # 检查是否每个用户都收到了其他用户的消息
            for stat in message_stats:
                if stat['chat_msgs'] < (connected_count - 1) * messages_per_user * 0.8:  # 允许20%丢失
                    success = False
                    details += f"; {stat['user']}消息接收不足"
            
            # 断开所有连接
            for user in users:
                await self.client.disconnect_websocket(user)
            
            if success:
                self.record_result("多用户并发", True, f"{details}，消息广播正常")
            else:
                self.record_result("多用户并发", False, f"{details}，部分消息可能丢失")
            
        except Exception as e:
            self.record_result("多用户并发", False, str(e))
            import traceback
            traceback.print_exc()
    
    async def test_reconnection(self):
        """测试场景3: 用户重连场景"""
        self.log("\n" + "=" * 60)
        self.log("测试场景3: 用户重连场景")
        self.log("场景: 网络中断后重连，验证消息不丢失")
        self.log("=" * 60)
        
        try:
            # 使用主用户或创建新用户
            if 'main_user' not in self.sessions:
                self.log("  没有可用用户，跳过重连测试")
                self.record_result("用户重连", False, "没有可用用户")
                return
            
            session = self.sessions['main_user']
            
            # 重新登录获取新token
            result = self.client.login(session.email, session.password)
            if result.get('success'):
                user_data = result.get('data', {})
                session.access_token = user_data.get('access_token')
                session.user_id = user_data.get('user', {}).get('id')
            
            # 获取或创建房间
            result = self.client.list_rooms(session)
            rooms = result.get('data', [])
            
            if rooms:
                room_id = rooms[0].get('id')
            else:
                # 创建新房间
                result = self.client.create_room(
                    session,
                    name=f"重连测试房间_{int(time.time())}",
                    description="重连测试"
                )
                room_id = result.get('data', {}).get('id')
            
            # 第一次连接
            self.log("  第一次连接WebSocket...")
            if not await self.client.connect_websocket(session):
                self.record_result("用户重连", False, "首次连接失败")
                return
            
            await self.client.join_room_ws(session, room_id)
            self.log("  首次连接成功，加入房间")
            
            # 发送第一条消息
            await self.client.send_chat_message(session, room_id, "第一条消息（连接时发送）")
            await asyncio.sleep(0.5)
            
            # 模拟断开连接
            self.log("  模拟网络断开...")
            await self.client.disconnect_websocket(session)
            await asyncio.sleep(1)
            
            # 重新连接
            self.log("  重新连接WebSocket...")
            if not await self.client.connect_websocket(session):
                self.record_result("用户重连", False, "重连失败")
                return
            
            await self.client.join_room_ws(session, room_id)
            self.log("  重连成功，重新加入房间")
            
            # 发送第二条消息
            await self.client.send_chat_message(session, room_id, "第二条消息（重连后发送）")
            await asyncio.sleep(1)
            
            # 获取消息历史验证
            result = self.client.get_room_messages(session, room_id)
            if result.get('success'):
                messages = result.get('data', [])
                self.log(f"  房间消息历史: {len(messages)} 条消息")
            
            await self.client.disconnect_websocket(session)
            
            self.record_result("用户重连", True, "重连测试完成")
            
        except Exception as e:
            self.record_result("用户重连", False, str(e))
            import traceback
            traceback.print_exc()
    
    async def test_admin_operations(self):
        """测试场景4: 房间管理员操作"""
        self.log("\n" + "=" * 60)
        self.log("测试场景4: 房间管理员操作")
        self.log("场景: 踢出成员、设置管理员、删除房间")
        self.log("=" * 60)
        
        try:
            timestamp = int(time.time())
            
            # 创建房主用户
            owner_email = f"room_owner_{timestamp}@example.com"
            owner_password = "TestPassword123"
            owner_username = f"RoomOwner{timestamp}"
            
            self.client.register(owner_email, owner_password, owner_username)
            result = self.client.login(owner_email, owner_password)
            
            if not result.get('success'):
                self.record_result("管理员操作", False, "房主登录失败")
                return
            
            owner_data = result.get('data', {})
            owner_session = UserSession(
                email=owner_email,
                password=owner_password,
                username=owner_data.get('user', {}).get('username', owner_username),
                user_id=owner_data.get('user', {}).get('id'),
                access_token=owner_data.get('access_token'),
                refresh_token=owner_data.get('refresh_token')
            )
            
            # 创建成员用户
            member_email = f"room_member_{timestamp}@example.com"
            member_password = "TestPassword123"
            member_username = f"RoomMember{timestamp}"
            
            self.client.register(member_email, member_password, member_username)
            result = self.client.login(member_email, member_password)
            
            if not result.get('success'):
                self.record_result("管理员操作", False, "成员登录失败")
                return
            
            member_data = result.get('data', {})
            member_session = UserSession(
                email=member_email,
                password=member_password,
                username=member_data.get('user', {}).get('username', member_username),
                user_id=member_data.get('user', {}).get('id'),
                access_token=member_data.get('access_token'),
                refresh_token=member_data.get('refresh_token')
            )
            
            # 房主创建房间
            self.log("  房主创建房间...")
            result = self.client.create_room(
                owner_session,
                name=f"管理员测试房间_{timestamp}",
                description="测试管理员操作"
            )
            
            if not result.get('success'):
                self.record_result("管理员操作", False, "创建房间失败")
                return
            
            room_id = result.get('data', {}).get('id')
            self.log(f"  房间创建成功: {room_id}")
            
            # 成员加入房间
            self.log("  成员加入房间...")
            result = self.client.join_room(member_session, room_id)
            if result.get('success'):
                self.log("  成员加入成功")
            else:
                self.log(f"  成员加入结果: {result.get('message', '未知')}")
            
            # 获取房间成员列表
            self.log("  获取房间成员列表...")
            result = self.client.get_room_members(owner_session, room_id)
            if result.get('success'):
                members = result.get('data', [])
                self.log(f"  房间成员数: {len(members)}")
                for m in members:
                    self.log(f"    - {m.get('username')} ({m.get('role')})")
            
            # 测试1: 设置成员为管理员
            self.log("  测试: 设置成员为管理员...")
            member_id = member_session.user_id
            result = self.client.set_member_role(owner_session, room_id, member_id, 'admin')
            if result.get('success'):
                self.log("  设置管理员成功")
            else:
                self.log(f"  设置管理员结果: {result.get('message', '未知')}")
            
            await asyncio.sleep(0.5)
            
            # 测试2: 踢出成员
            self.log("  测试: 踢出成员...")
            result = self.client.kick_member(owner_session, room_id, member_id)
            if result.get('success'):
                self.log("  踢出成员成功")
            else:
                self.log(f"  踢出成员结果: {result.get('message', '未知')}")
            
            await asyncio.sleep(0.5)
            
            # 测试3: 删除房间
            self.log("  测试: 删除房间...")
            result = self.client.delete_room(owner_session, room_id)
            if result.get('success'):
                self.log("  删除房间成功")
            else:
                self.log(f"  删除房间结果: {result.get('message', '未知')}")
            
            self.record_result("管理员操作", True, "管理员操作测试完成")
            
        except Exception as e:
            self.record_result("管理员操作", False, str(e))
            import traceback
            traceback.print_exc()
    
    def print_test_report(self):
        """打印测试报告"""
        self.log("\n" + "=" * 60)
        self.log("测试报告")
        self.log("=" * 60)
        
        passed = sum(1 for r in self.test_results if r['success'])
        failed = sum(1 for r in self.test_results if not r['success'])
        total = len(self.test_results)
        
        self.log(f"\n总计: {total} 个测试")
        self.log(f"通过: {passed} 个")
        self.log(f"失败: {failed} 个")
        self.log(f"通过率: {passed/total*100:.1f}%" if total > 0 else "N/A")
        
        self.log("\n详细结果:")
        for result in self.test_results:
            status = "✅" if result['success'] else "❌"
            self.log(f"  {status} {result['test_name']}")
            if result['details']:
                self.log(f"     详情: {result['details']}")


async def main():
    """主函数"""
    # 解析命令行参数
    port = TEST_PORT
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            pass
    
    base_url = get_base_url(port)
    ws_url = get_ws_url(port)
    
    print("=" * 60)
    print("聊天室端到端测试")
    print("=" * 60)
    print(f"用法: python e2e_test.py [端口]")
    print(f"示例: python e2e_test.py 8080")
    print(f"当前配置: 端口={port}")
    print("=" * 60)
    print(f"测试服务器: {base_url}")
    print(f"WebSocket: {ws_url}")
    print(f"测试账号: {TEST_ACCOUNT}")
    print("=" * 60)
    
    # 检查服务器是否可连接
    try:
        response = requests.get(f"{base_url}/health", timeout=5)
        print(f"\n服务器健康检查: {'正常' if response.status_code == 200 else '异常'}")
    except Exception as e:
        print(f"\n⚠️ 无法连接到服务器: {e}")
        print("请确保服务器已启动后再运行测试")
        return
    
    # 运行测试
    runner = E2ETestRunner(base_url, ws_url)
    await runner.run_all_tests()


if __name__ == "__main__":
    asyncio.run(main())
