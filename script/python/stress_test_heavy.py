"""
高负载压力测试 - 100,000 条消息收发

策略: 500 并发用户 × 200 条消息/人，多房间分片
目标: 验证批次写入、广播扇出、内存稳定性在大流量下的表现

用法:
    sudo .venv/bin/python stress_test_heavy.py
"""

import asyncio
import aiohttp
import websockets
import json, time, sys, os, subprocess, statistics
from dataclasses import dataclass, field
from typing import List, Optional, Dict
from dotenv import load_dotenv

load_dotenv('../.env')

HOST = os.getenv('TEST_HOST', 'localhost')
PORT = int(os.getenv('TEST_PORT', '3000'))
BASE = f"http://{HOST}:{PORT}"
WS_URL = f"ws://{HOST}:{PORT}/ws"

TEST_USERS = [
    {'email': f'TestUser{i}@test.com', 'password': 'Test12345'}
    for i in range(3, 103)
]

@dataclass
class Metrics:
    user_id: str
    connect_time: float = 0
    sent: int = 0
    received: int = 0
    errors: List[str] = field(default_factory=list)

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

class Client:
    def __init__(self):
        self.session: Optional[aiohttp.ClientSession] = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, *a):
        if self.session:
            await self.session.close()

    async def login(self, email: str, password: str) -> Optional[str]:
        try:
            async with self.session.post(f"{BASE}/api/auth/login",
                json={"email": email, "password": password}) as resp:
                if resp.status == 200:
                    return (await resp.json()).get('data', {}).get('access_token')
        except: pass
        return None

    async def create_room(self, token: str, name: str, max_m: int) -> Optional[str]:
        try:
            async with self.session.post(f"{BASE}/api/rooms/",
                headers={"Authorization": f"Bearer {token}"},
                json={"name": name, "description": "压力测试",
                      "is_private": False, "max_members": max_m}) as resp:
                if resp.status in (200, 201):
                    return (await resp.json()).get('data', {}).get('id')
        except: pass
        return None

    async def ws_connect(self, token: str) -> Optional:
        try:
            ws = await websockets.connect(WS_URL,
                additional_headers={"Authorization": f"Bearer {token}"},
                open_timeout=10, close_timeout=5)
            auth_msg = {"type": "Auth", "payload": {"token": token}}
            await ws.send(json.dumps(auth_msg))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            if resp.get('type') == 'AuthResult' and resp.get('payload', {}).get('success'):
                return ws
            await ws.close()
        except: pass
        return None


async def user_worker(client: Client, email: str, password: str,
                      room_id: str, msg_count: int,
                      sem: asyncio.Semaphore, metrics: Metrics):
    async with sem:
        t0 = time.time()
        ws = None
        try:
            token = await client.login(email, password)
            if not token:
                metrics.errors.append(f"login_fail:{email}")
                return
            ws = await client.ws_connect(token)
            if not ws:
                metrics.errors.append(f"ws_fail:{email}")
                return
            metrics.connect_time = time.time() - t0

            # Join room
            await ws.send(json.dumps({"type": "JoinRoom", "payload": {"room_id": room_id}}))
            joined = False
            for _ in range(10):
                try:
                    msg = json.loads(await asyncio.wait_for(ws.recv(), timeout=2))
                    if msg.get('type') == 'RoomJoined':
                        joined = True; break
                    elif msg.get('type') == 'Ping':
                        await ws.send(json.dumps({"type": "Pong"}))
                except: break
            if not joined:
                metrics.errors.append(f"join_fail:{email}")
                return

            # Send messages
            for i in range(msg_count):
                try:
                    await ws.send(json.dumps({
                        "type": "ChatMessage",
                        "payload": {
                            "room_id": room_id,
                            "content": f"heavy-{email}-{i}"
                        }
                    }))
                    metrics.sent += 1
                    # Read incoming (own broadcast + others)
                    try:
                        r = await asyncio.wait_for(ws.recv(), timeout=1.0)
                        metrics.received += 1
                    except: pass
                except:
                    metrics.errors.append(f"send_fail:{email}:{i}")
                    break

            # Drain remaining messages
            await asyncio.sleep(1)
            while True:
                try:
                    await asyncio.wait_for(ws.recv(), timeout=0.3)
                    metrics.received += 1
                except: break

        finally:
            if ws: await ws.close()


async def main():
    log("="*60)
    log(f"高负载压力测试: 100,000 条消息")
    log(f"服务器: {BASE}")
    log("="*60)

    sem = asyncio.Semaphore(30)  # 30 concurrent connections

    async with Client() as c:
        # Create rooms
        admin_token = await c.login("admin@example.com", "admin123456")
        rooms = []
        for i in range(5):
            rid = await c.create_room(admin_token, f"heavy-test-{i}", 200)
            if rid:
                rooms.append(rid)
                log(f"  房间 {i+1}: {rid[:8]}...")

        if not rooms:
            log("❌ 房间创建失败"); return

        CONCURRENT = 500
        MSG_PER_USER = 200
        TOTAL_MSGS = CONCURRENT * MSG_PER_USER

        # Assign rooms round-robin
        assignments = [rooms[i % len(rooms)] for i in range(CONCURRENT)]

        # Get memory before
        mem_before = 0
        try:
            r = subprocess.run(['docker', 'stats', '--no-stream', 'capella-room-prod'],
                             capture_output=True, text=True)
            parts = r.stdout.split('\n')[1].split()
            for p in parts:
                if 'MiB' in p: mem_before = float(p.replace('MiB','')); break
                elif 'GiB' in p: mem_before = float(p.replace('GiB','')) * 1024; break
        except: pass
        log(f"测试前内存: {mem_before:.0f} MiB" if mem_before else "内存监控: N/A")

        # Run concurrent users
        all_metrics = []
        t_start = time.time()
        tasks = []
        for i in range(CONCURRENT):
            u = TEST_USERS[i % len(TEST_USERS)]
            m = Metrics(user_id=f"{u['email']}#{i}")
            all_metrics.append(m)
            tasks.append(user_worker(c, u['email'], u['password'],
                          assignments[i], MSG_PER_USER, sem, m))

        await asyncio.gather(*tasks, return_exceptions=True)
        elapsed = time.time() - t_start

        # Memory after
        await asyncio.sleep(2)
        mem_after = 0
        try:
            r = subprocess.run(['docker', 'stats', '--no-stream', 'capella-room-prod'],
                             capture_output=True, text=True)
            parts = r.stdout.split('\n')[1].split()
            for p in parts:
                if 'MiB' in p: mem_after = float(p.replace('MiB','')); break
                elif 'GiB' in p: mem_after = float(p.replace('GiB','')) * 1024; break
        except: pass

        # Stats
        total_sent = sum(m.sent for m in all_metrics)
        total_recv = sum(m.received for m in all_metrics)
        connect_times = [m.connect_time for m in all_metrics if m.connect_time > 0]
        all_errors = []
        for m in all_metrics: all_errors.extend(m.errors)
        success = sum(1 for m in all_metrics if m.sent > 0)
        fail = CONCURRENT - success
        avg_ct = statistics.mean(connect_times) if connect_times else 0
        max_ct = max(connect_times) if connect_times else 0
        msg_rate = total_sent / elapsed if elapsed > 0 else 0

        log("\n" + "="*60)
        log("结果")
        log("="*60)
        log(f"  目标:           {TOTAL_MSGS} 条 ({CONCURRENT}用户 × {MSG_PER_USER}条)")
        log(f"  实际发送:       {total_sent} 条")
        log(f"  实际接收:       {total_recv} 条")
        log(f"  成功率:         {success}/{CONCURRENT} ({success/CONCURRENT*100:.1f}%)")
        log(f"  消息发送速率:   {msg_rate:.0f} msg/s")
        log(f"  总耗时:         {elapsed:.1f}s")
        log(f"  平均连接:       {avg_ct:.3f}s")
        log(f"  最大连接:       {max_ct:.3f}s")
        log(f"  内存:           {mem_before:.0f} → {mem_after:.0f} MiB ({mem_after-mem_before:+.0f})" if mem_after else "")
        log(f"  错误:           {len(all_errors)}")
        if all_errors:
            from collections import Counter
            reasons = Counter(e.split(':')[0] for e in all_errors)
            for r, c in reasons.most_common(5):
                log(f"    {r}: {c}")
        log(f"  吞吐:           {total_sent/elapsed:.0f} msg/s | {total_recv/elapsed:.0f} recv/s")

if __name__ == '__main__':
    asyncio.run(main())
