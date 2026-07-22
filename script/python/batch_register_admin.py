"""批量注册测试用户（admin token 方式）
用法: uv run python batch_register_admin.py [起始编号] [数量]
"""
import requests, sys

BASE = "http://127.0.0.1:3000"
PASSWORD = "Test12345"

# 1. admin 登录获取 token
r = requests.post(f"{BASE}/api/auth/login", json={
    "email": "admin@example.com",
    "password": "admin123456"
})
token = r.json()["data"]["access_token"]
headers = {"Authorization": f"Bearer {token}"}
print(f"Admin token: {token[:20]}...")

# 2. 注册用户
start = int(sys.argv[1]) if len(sys.argv) > 1 else 3
count = int(sys.argv[2]) if len(sys.argv) > 2 else 100

ok = 0
for i in range(start, start + count):
    email = f"TestUser{i}@test.com"
    username = f"TestUser{i}"
    r = requests.post(f"{BASE}/api/auth/register",
        headers=headers,
        json={"email": email, "password": PASSWORD, "username": username})
    if r.json().get("success"):
        ok += 1
        print(f"  [{i}] {email} ✅")
    else:
        msg = r.json().get("message", "")
        if "已被注册" in msg:
            ok += 1
            print(f"  [{i}] {email} ⚠️ 已存在")
        else:
            print(f"  [{i}] {email} ❌ {msg}")

print(f"\n完成: {ok}/{count}")
