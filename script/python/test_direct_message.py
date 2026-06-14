"""
Test direct message workflow:
1. Login as TestUser2
2. Create a direct room with TestUser1
3. Send a message in that room via WebSocket
"""

import requests
import json
import websocket
import time

BASE_URL = "http://localhost:8080"
WS_URL = "ws://localhost:8080/ws"
PASSWORD = "Test12345"

def login(email):
    resp = requests.post(f"{BASE_URL}/api/v1/auth/login", json={
        "email": email,
        "password": PASSWORD,
    })
    data = resp.json()
    print(f"[Login] {email}: {'OK' if data.get('success') else 'FAIL'}")
    if not data.get('success'):
        print(f"  Error: {data.get('message')}")
        return None
    return data['data']['access_token']

TESTUSER1_ID = "e1a7b834-f7f9-4e0a-95d6-5a59df374e55"

def create_direct_room(token, target_user_id):
    resp = requests.post(
        f"{BASE_URL}/api/v1/rooms/direct",
        headers={"Authorization": f"Bearer {token}"},
        json={"target_user_id": target_user_id}
    )
    data = resp.json()
    print(f"[CreateDirectRoom] -> {target_user_id}: {'OK' if data.get('success') else 'FAIL'}")
    if not data.get('success'):
        print(f"  Error: {data.get('message')}")
        print(f"  Full response: {data}")
        return None
    print(f"  Room ID: {data['data'].get('id')}")
    return data['data']['id']

def send_message_via_ws(token, room_id, content):
    ws = websocket.create_connection(WS_URL)
    # Auth
    ws.send(json.dumps({"type": "Auth", "payload": {"token": token}}))
    time.sleep(0.5)
    # Join room
    ws.send(json.dumps({"type": "JoinRoom", "payload": {"room_id": room_id}}))
    time.sleep(0.5)
    # Send message
    ws.send(json.dumps({
        "type": "ChatMessage",
        "payload": {
            "room_id": room_id,
            "content": content,
            "reply_to": None
        }
    }))
    time.sleep(1)
    # Collect responses
    msgs = []
    ws.settimeout(2)
    while True:
        try:
            msg = ws.recv()
            msgs.append(json.loads(msg))
        except:
            break
    ws.close()
    print(f"[WebSocket] Sent message, received {len(msgs)} responses")
    for m in msgs:
        print(f"  <- type={m.get('type')}, payload={json.dumps(m.get('payload'), ensure_ascii=False)[:120]}")
    return msgs

# Step 1: Login as TestUser2
token = login("TestUser2@test.com")
if not token:
    exit(1)

# Step 2: Create direct room with TestUser1 by UUID
room_id = create_direct_room(token, TESTUSER1_ID)
if not room_id:
    exit(1)

# Step 4: Send a message
send_message_via_ws(token, room_id, "Hello from TestUser2! This is a direct message test. 👋")

print("\n[Done] TestUser1 should now see a direct room from TestUser2 in their chat list.")
