#!/usr/bin/env python3
"""
验证邮箱注册流程——结合 QQ 邮箱 SMTP 发验证码
"""
import requests, json, sys

BASE = "http://127.0.0.1:3000"
EMAIL = sys.argv[1] if len(sys.argv) > 1 else "2133765105@qq.com"

# Step 1: 发送验证码
print(f"📧 向 {EMAIL} 发送注册验证码...")
r = requests.post(f"{BASE}/api/v2/auth/register/send-code",
    json={"email": EMAIL})
data = r.json()
print(f"   {data}")
if not data.get("success"):
    print("❌ 验证码发送失败，检查 SMTP 配置")
    sys.exit(1)

# Step 2: 用户输入验证码
code = input("✏️  请输入邮箱中收到的 6 位验证码: ").strip()

# Step 3: 注册
username = f"user_{EMAIL.split('@')[0]}"
password = "TestPass123!"
print(f"\n📝 注册账号: {username} / {EMAIL}")
r = requests.post(f"{BASE}/api/v2/auth/register", json={
    "email": EMAIL,
    "code": code,
    "username": username,
    "password": password,
})
data = r.json()
if data.get("success"):
    print(f"✅ 注册成功！")
    print(f"   用户名: {username}")
    print(f"   密码:   {password}")
    token = data.get("data", {}).get("access_token", "")[:20]
    print(f"   Token:  {token}...")
else:
    print(f"❌ 注册失败: {data.get('message', data)}")
