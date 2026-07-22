"""
极限吞吐压测 - 测系统纯消息吞吐上限

策略: 500用户 × 500条/人，不等待广播回包（只关心发送+入库）
"""

import asyncio, aiohttp, websockets, json, time, os, subprocess
from dotenv import load_dotenv
load_dotenv('../.env')

HOST, PORT = os.getenv('TEST_HOST', 'localhost'), int(os.getenv('TEST_PORT', '3000'))
BASE, WS_URL = f"http://{HOST}:{PORT}", f"ws://{HOST}:{PORT}/ws"
USERS = [{'email': f'TestUser{i}@test.com', 'password': 'Test12345'} for i in range(3, 103)]

def log(m): print(f"[{time.strftime('%H:%M:%S')}] {m}")

async def worker(client, email, pw, room_id, n, sem, results):
    async with sem:
        t0 = time.time()
        try:
            r = await client.session.post(f"{BASE}/api/auth/login",
                json={"email": email, "password": pw})
            token = (await r.json()).get('data', {}).get('access_token')
            if not token: return

            ws = await websockets.connect(WS_URL,
                additional_headers={"Authorization": f"Bearer {token}"},
                open_timeout=10, close_timeout=5)
            await ws.send(json.dumps({"type":"Auth","payload":{"token":token}}))
            await asyncio.wait_for(ws.recv(), 5)  # AuthResult

            await ws.send(json.dumps({"type":"JoinRoom","payload":{"room_id":room_id}}))
            for _ in range(10):
                m = json.loads(await asyncio.wait_for(ws.recv(), 2))
                if m.get('type') == 'RoomJoined': break
                elif m.get('type') == 'Ping': await ws.send(json.dumps({"type":"Pong"}))

            for i in range(n):
                await ws.send(json.dumps({
                    "type":"ChatMessage",
                    "payload":{"room_id":room_id, "content":f"max-{email}-{i}"}
                }))
            await ws.close()
            results['sent'] += n
            results['ok'] += 1
        except Exception as e:
            results['err'] += 1

async def main():
    log("="*60)
    log("极限吞吐压测: 500用户 × 500条 = 250,000 消息")
    log(f"batch_size=2000, CPU=8核, WS不等待广播回包")
    log("="*60)

    sem = asyncio.Semaphore(50)
    results = {'sent': 0, 'ok': 0, 'err': 0}

    # 查内存
    mem_before = 0
    try:
        r = subprocess.run(['docker','stats','--no-stream','capella-room-prod'],
                          capture_output=True,text=True)
        for p in r.stdout.split('\n')[1].split():
            if 'MiB' in p: mem_before=float(p.replace('MiB','')); break
            elif 'GiB' in p: mem_before=float(p.replace('GiB',''))*1024; break
    except: pass

    async with aiohttp.ClientSession() as session:
        client = type('',(),{'session':session})()
        # 建一个房间
        r = await session.post(f"{BASE}/api/auth/login",
            json={"email":"admin@example.com","password":"admin123456"})
        t = (await r.json())['data']['access_token']
        r = await session.post(f"{BASE}/api/rooms/",
            headers={"Authorization":f"Bearer {t}"},
            json={"name":"max-test","description":"","is_private":False,"max_members":200})
        room = (await r.json()).get('data',{}).get('id')

        # 500用户 × 500条
        t0 = time.time()
        tasks = []
        for i in range(500):
            u = USERS[i % len(USERS)]
            tasks.append(worker(client, u['email'], u['password'], room, 500, sem, results))
        await asyncio.gather(*tasks, return_exceptions=True)
        elapsed = time.time() - t0

    await asyncio.sleep(2)
    mem_after = 0
    try:
        r = subprocess.run(['docker','stats','--no-stream','capella-room-prod'],
                          capture_output=True,text=True)
        for p in r.stdout.split('\n')[1].split():
            if 'MiB' in p: mem_after=float(p.replace('MiB','')); break
            elif 'GiB' in p: mem_after=float(p.replace('GiB',''))*1024; break
    except: pass

    log("\n"+ "="*60)
    log("结果")
    log("="*60)
    log(f"  目标:       250,000 条 (500用户 × 500条)")
    log(f"  实际发送:   {results['sent']} 条")
    log(f"  成功率:     {results['ok']}/{results['ok']+results['err']} ({results['ok']/(results['ok']+results['err'])*100:.1f}%)")
    log(f"  发送速率:   {results['sent']/elapsed:.0f} msg/s")
    log(f"  总耗时:     {elapsed:.1f}s")
    log(f"  连接建立:   {elapsed/(500 if results['ok']>0 else 1):.2f}s/用户")
    log(f"  内存:       {mem_before:.0f} → {mem_after:.0f} MiB ({mem_after-mem_before:+.0f})")

if __name__ == '__main__':
    asyncio.run(main())
