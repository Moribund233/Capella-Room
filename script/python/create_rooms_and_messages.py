"""
创建测试房间并模拟用户互发消息

使用方式:
    python create_rooms_and_messages.py [端口]
    
示例:
    python create_rooms_and_messages.py           # 使用默认端口8765
    python create_rooms_and_messages.py 8080      # 使用端口8080
    
环境变量:
    TEST_PORT: 服务器端口 (默认8765)

功能:
1. 创建多个测试房间
2. 使用10个测试账号加入房间
3. 模拟用户互发100条随机消息
"""

import requests
import json
import time
import random
import sys
import os
from typing import List, Dict, Optional
from datetime import datetime

# 配置
DEFAULT_PORT = int(os.getenv('TEST_PORT', '8765'))
DEFAULT_PASSWORD = "Test12345"

# 随机消息内容模板
MESSAGE_TEMPLATES = [
    "大家好！",
    "今天天气不错",
    "有人在吗？",
    "这个房间真热闹",
    "我是新来的",
    "测试消息 {}",
    "哈哈，真有趣",
    "有人知道怎么用吗？",
    "欢迎欢迎！",
    "我也来凑个热闹",
    "这个功能不错",
    "期待更多功能",
    "有人在线吗？",
    "测试一下消息功能",
    "随机消息 {}",
    "Hello everyone!",
    "Nice to meet you all",
    "这个房间有多少人？",
    "我来发个消息",
    "测试完成！",
]


def get_base_url(port: int) -> str:
    """获取基础URL"""
    return f"http://localhost:{port}"


def login_user(base_url: str, email: str, password: str) -> Optional[str]:
    """
    用户登录获取 token
    
    Args:
        base_url: 基础URL
        email: 邮箱
        password: 密码
        
    Returns:
        JWT access_token 或 None
    """
    url = f"{base_url}/api/v1/auth/login"
    data = {
        'email': email,
        'password': password
    }
    
    try:
        response = requests.post(url, json=data, timeout=10)
        result = response.json()
        if result.get('success'):
            # API 返回的是 access_token 而不是 token
            return result.get('data', {}).get('access_token')
        else:
            print(f"登录失败 {email}: {result.get('message')}")
            return None
    except Exception as e:
        print(f"登录异常 {email}: {e}")
        return None


def create_room(base_url: str, token: str, name: str, description: str = "", is_private: bool = False, max_members: int = 50) -> Optional[str]:
    """
    创建聊天室
    
    Args:
        base_url: 基础URL
        token: JWT token
        name: 房间名称
        description: 房间描述
        is_private: 是否私有
        max_members: 最大成员数
        
    Returns:
        房间ID 或 None
    """
    url = f"{base_url}/api/v1/rooms"
    headers = {'Authorization': f'Bearer {token}'}
    data = {
        'name': name,
        'description': description,
        'is_private': is_private,
        'max_members': max_members
    }
    
    try:
        response = requests.post(url, json=data, headers=headers, timeout=10)
        result = response.json()
        if result.get('success'):
            room_id = result.get('data', {}).get('id')
            print(f"  ✅ 创建房间成功: {name} (ID: {room_id})")
            return room_id
        else:
            print(f"  ❌ 创建房间失败 {name}: {result.get('message')}")
            return None
    except Exception as e:
        print(f"  ❌ 创建房间异常 {name}: {e}")
        return None


def join_room(base_url: str, token: str, room_id: str) -> bool:
    """
    加入聊天室
    
    Args:
        base_url: 基础URL
        token: JWT token
        room_id: 房间ID
        
    Returns:
        是否成功
    """
    url = f"{base_url}/api/v1/rooms/{room_id}/join"
    headers = {'Authorization': f'Bearer {token}'}
    
    try:
        response = requests.post(url, headers=headers, timeout=10)
        result = response.json()
        if result.get('success') or '已加入' in result.get('message', ''):
            return True
        else:
            print(f"    加入房间失败: {result.get('message')}")
            return False
    except Exception as e:
        print(f"    加入房间异常: {e}")
        return False


def send_message_ws(ws_url: str, token: str, room_id: str, content: str) -> bool:
    """
    通过 WebSocket 发送消息
    
    Args:
        ws_url: WebSocket URL
        token: JWT token
        room_id: 房间ID
        content: 消息内容
        
    Returns:
        是否成功
    """
    import websocket
    import json
    
    try:
        # 建立 WebSocket 连接
        ws = websocket.create_connection(ws_url, timeout=10)
        
        # 发送认证消息
        auth_msg = {
            'type': 'Auth',
            'payload': {'token': token}
        }
        ws.send(json.dumps(auth_msg))
        
        # 等待认证结果
        response = ws.recv()
        resp_data = json.loads(response)
        if resp_data.get('type') != 'AuthResult' or not resp_data.get('payload', {}).get('success'):
            ws.close()
            return False
        
        # 加入房间
        join_msg = {
            'type': 'JoinRoom',
            'payload': {'room_id': room_id}
        }
        ws.send(json.dumps(join_msg))
        
        # 等待加入确认
        ws.recv()
        
        # 发送消息
        chat_msg = {
            'type': 'ChatMessage',
            'payload': {
                'room_id': room_id,
                'content': content
            }
        }
        ws.send(json.dumps(chat_msg))
        
        ws.close()
        return True
    except Exception as e:
        print(f"    发送消息异常: {e}")
        return False


def get_user_info(base_url: str, token: str) -> Optional[Dict]:
    """
    获取用户信息
    
    Args:
        base_url: 基础URL
        token: JWT token
        
    Returns:
        用户信息 或 None
    """
    url = f"{base_url}/api/v1/users/me"
    headers = {'Authorization': f'Bearer {token}'}
    
    try:
        response = requests.get(url, headers=headers, timeout=10)
        result = response.json()
        if result.get('success'):
            return result.get('data')
        return None
    except Exception as e:
        print(f"获取用户信息异常: {e}")
        return None


def create_test_rooms(base_url: str, creator_token: str, count: int = 5) -> List[str]:
    """
    创建测试房间
    
    Args:
        base_url: 基础URL
        creator_token: 创建者token
        count: 房间数量
        
    Returns:
        房间ID列表
    """
    room_names = [
        ("技术交流", "讨论各种技术话题"),
        ("闲聊灌水", "随便聊聊，放松心情"),
        ("游戏开黑", "一起玩游戏"),
        ("音乐分享", "分享好听的音乐"),
        ("电影推荐", "推荐好看的电影"),
        ("读书交流", "分享读书心得"),
        ("美食探店", "分享美食和餐厅"),
        ("旅行攻略", "分享旅行经验"),
    ]
    
    room_ids = []
    print("\n" + "=" * 60)
    print("创建测试房间")
    print("=" * 60)
    
    for i in range(min(count, len(room_names))):
        name, description = room_names[i]
        is_private = random.choice([True, False])
        max_members = random.choice([20, 50, 100])
        
        print(f"[{i+1}/{count}] 创建房间: {name} (私有: {is_private}, 最大成员: {max_members})")
        room_id = create_room(base_url, creator_token, name, description, is_private, max_members)
        if room_id:
            room_ids.append(room_id)
        time.sleep(0.3)
    
    return room_ids


def simulate_user_interactions(base_url: str, ws_url: str, room_ids: List[str], user_tokens: List[str], message_count: int = 100):
    """
    模拟用户互发消息
    
    Args:
        base_url: 基础URL
        ws_url: WebSocket URL
        room_ids: 房间ID列表
        user_tokens: 用户token列表
        message_count: 消息数量
    """
    print("\n" + "=" * 60)
    print("模拟用户互发消息")
    print("=" * 60)
    print(f"房间数: {len(room_ids)}, 用户数: {len(user_tokens)}, 目标消息数: {message_count}")
    print("=" * 60)
    
    # 获取用户信息
    users_info = []
    for i, token in enumerate(user_tokens):
        user_info = get_user_info(base_url, token)
        if user_info:
            users_info.append({
                'token': token,
                'email': user_info.get('email', f'User{i}'),
                'username': user_info.get('username', f'User{i}'),
                'id': user_info.get('id', '')
            })
    
    print(f"成功获取 {len(users_info)} 个用户信息")
    
    # 每个用户加入所有房间
    print("\n用户加入房间...")
    for user in users_info:
        joined_count = 0
        for room_id in room_ids:
            if join_room(base_url, user['token'], room_id):
                joined_count += 1
            time.sleep(0.1)
        print(f"  {user['username']}: 加入 {joined_count}/{len(room_ids)} 个房间")
    
    # 随机发送消息
    print(f"\n开始发送 {message_count} 条消息...")
    success_count = 0
    failed_count = 0
    
    for i in range(message_count):
        # 随机选择用户和房间
        user = random.choice(users_info)
        room_id = random.choice(room_ids)
        
        # 随机生成消息内容
        template = random.choice(MESSAGE_TEMPLATES)
        if '{}' in template:
            content = template.format(random.randint(1, 999))
        else:
            content = template
        
        # 发送消息
        if send_message_ws(ws_url, user['token'], room_id, content):
            success_count += 1
            print(f"  [{i+1}/{message_count}] ✅ {user['username']}: {content[:30]}...")
        else:
            failed_count += 1
            print(f"  [{i+1}/{message_count}] ❌ {user['username']}: 发送失败")
        
        # 随机延迟，模拟真实场景
        time.sleep(random.uniform(0.1, 0.5))
    
    print(f"\n消息发送完成: 成功 {success_count}, 失败 {failed_count}")


def get_ws_url(port: int) -> str:
    """获取WebSocket URL"""
    return f"ws://localhost:{port}/ws"


def main():
    """主函数"""
    # 默认端口
    port = DEFAULT_PORT
    
    # 解析命令行参数
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            pass
    
    base_url = get_base_url(port)
    ws_url = get_ws_url(port)
    
    print("=" * 60)
    print("创建测试房间并模拟用户互发消息")
    print("=" * 60)
    print(f"服务器端口: {port}")
    print(f"基础URL: {base_url}")
    print(f"WebSocket: {ws_url}")
    print()
    
    # 检查服务器连接
    try:
        response = requests.get(f"{base_url}/health", timeout=5)
        print(f"服务器健康检查: {'正常' if response.status_code == 200 else '异常'}")
    except Exception as e:
        print(f"⚠️ 无法连接到服务器: {e}")
        print("请确保服务器已启动后再运行脚本")
        return
    
    # 使用 TestUser1-10 登录
    print("\n登录测试用户...")
    user_tokens = []
    for i in range(1, 11):
        email = f"TestUser{i}@test.com"
        print(f"  登录 {email} ...", end=" ")
        token = login_user(base_url, email, DEFAULT_PASSWORD)
        if token:
            print("✅ 成功")
            user_tokens.append(token)
        else:
            print("❌ 失败")
        time.sleep(0.2)
    
    if len(user_tokens) < 2:
        print("⚠️ 成功登录的用户数不足2个，无法继续测试")
        return
    
    print(f"\n成功登录 {len(user_tokens)} 个用户")
    
    # 使用第一个用户创建房间
    creator_token = user_tokens[0]
    room_ids = create_test_rooms(base_url, creator_token, count=5)
    
    if not room_ids:
        print("⚠️ 没有成功创建任何房间，无法继续测试")
        return
    
    print(f"\n成功创建 {len(room_ids)} 个房间")
    
    # 模拟用户互发消息
    simulate_user_interactions(base_url, ws_url, room_ids, user_tokens, message_count=100)
    
    # 打印总结
    print("\n" + "=" * 60)
    print("测试完成总结")
    print("=" * 60)
    print(f"参与用户: 10 个 (TestUser1-10)")
    print(f"创建房间: {len(room_ids)} 个")
    print(f"发送消息: 100 条")
    print("\n房间列表:")
    for i, room_id in enumerate(room_ids, 1):
        print(f"  {i}. {room_id}")
    print("\n现在可以在管理后台验证:")
    print("  1. 房间管理 - 查看房间列表和详情")
    print("  2. 消息管理 - 查看各房间的消息")
    print("=" * 60)


if __name__ == "__main__":
    main()
