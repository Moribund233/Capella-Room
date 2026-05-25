"""
创建测试房间并模拟用户互发消息

使用方式:
    python create_rooms_and_messages.py [用户数量] [消息数量] [端口]

示例:
    python create_rooms_and_messages.py              # 使用默认：100用户，100000消息，端口3000
    python create_rooms_and_messages.py 100 100000   # 100用户，100000消息
    python create_rooms_and_messages.py 100 100000 3000  # 指定端口

环境变量:
    TEST_PORT: 服务器端口 (默认3000)

功能:
1. 创建多个测试房间（只创建公开房间）
2. 使用测试账号加入房间
3. 模拟用户互发消息（高并发，低延迟）
"""

import requests
import json
import time
import random
import sys
import os
from typing import List, Dict, Optional, Tuple
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading

# 配置
DEFAULT_PORT = int(os.getenv('TEST_PORT', '3000'))
DEFAULT_PASSWORD = "Test12345"
DEFAULT_USER_COUNT = 100
DEFAULT_MESSAGE_COUNT = 100000

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
    "有人想聊天吗？",
    "今天过得怎么样？",
    "有什么好玩的？",
    "新人报道！",
    "大佬带带我",
]

# 线程本地存储
thread_local = threading.local()

def get_base_url(port: int) -> str:
    """获取基础URL"""
    return f"http://localhost:{port}"


def login_user(base_url: str, email: str, password: str) -> Optional[str]:
    """用户登录获取 token"""
    url = f"{base_url}/api/v1/auth/login"
    data = {'email': email, 'password': password}
    
    try:
        response = requests.post(url, json=data, timeout=10)
        result = response.json()
        if result.get('success'):
            return result.get('data', {}).get('access_token')
        return None
    except Exception:
        return None


def create_room(base_url: str, token: str, name: str, description: str = "", is_private: bool = False, max_members: int = 200) -> Optional[Tuple[str, bool]]:
    """创建聊天室，返回 (房间ID, 是否私有)"""
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
            print(f"  ✅ 创建房间成功: {name} (ID: {room_id}, 私有: {is_private})")
            return (room_id, is_private)
        else:
            print(f"  ❌ 创建房间失败 {name}: {result.get('message')}")
            return None
    except Exception as e:
        print(f"  ❌ 创建房间异常 {name}: {e}")
        return None


def join_room(base_url: str, token: str, room_id: str) -> bool:
    """加入聊天室"""
    url = f"{base_url}/api/v1/rooms/{room_id}/join"
    headers = {'Authorization': f'Bearer {token}'}
    
    try:
        response = requests.post(url, headers=headers, timeout=5)
        result = response.json()
        return result.get('success') or '已加入' in result.get('message', '')
    except Exception:
        return False


def get_user_info(base_url: str, token: str) -> Optional[Dict]:
    """获取用户信息"""
    url = f"{base_url}/api/v1/users/me"
    headers = {'Authorization': f'Bearer {token}'}
    
    try:
        response = requests.get(url, headers=headers, timeout=5)
        result = response.json()
        if result.get('success'):
            return result.get('data')
        return None
    except Exception:
        return None


def send_message_batch(ws_url: str, user_token: str, room_id: str, messages: List[str]) -> int:
    """批量发送消息，使用单个WebSocket连接"""
    import websocket
    
    success_count = 0
    try:
        ws = websocket.create_connection(ws_url, timeout=10)
        
        # 认证
        auth_msg = {'type': 'Auth', 'payload': {'token': user_token}}
        ws.send(json.dumps(auth_msg))
        ws.recv()  # 等待认证结果
        
        # 加入房间
        join_msg = {'type': 'JoinRoom', 'payload': {'room_id': room_id}}
        ws.send(json.dumps(join_msg))
        ws.recv()  # 等待加入确认
        
        # 批量发送消息
        for content in messages:
            chat_msg = {
                'type': 'ChatMessage',
                'payload': {'room_id': room_id, 'content': content}
            }
            ws.send(json.dumps(chat_msg))
            success_count += 1
        
        ws.close()
    except Exception:
        pass
    
    return success_count


def create_test_rooms(base_url: str, creator_token: str, count: int = 10) -> List[Tuple[str, bool]]:
    """创建测试房间，只创建公开房间"""
    room_names = [
        ("技术交流", "讨论各种技术话题"),
        ("闲聊灌水", "随便聊聊，放松心情"),
        ("游戏开黑", "一起玩游戏"),
        ("音乐分享", "分享好听的音乐"),
        ("电影推荐", "推荐好看的电影"),
        ("读书交流", "分享读书心得"),
        ("美食探店", "分享美食和餐厅"),
        ("旅行攻略", "分享旅行经验"),
        ("摄影交流", "摄影技巧分享"),
        ("运动健身", "运动健身打卡"),
        ("宠物天地", "分享萌宠日常"),
        ("职场交流", "职场经验分享"),
    ]
    
    rooms = []
    print("\n" + "=" * 60)
    print("创建测试房间（仅公开房间）")
    print("=" * 60)
    
    for i in range(min(count, len(room_names))):
        name, description = room_names[i]
        # 只创建公开房间
        result = create_room(base_url, creator_token, name, description, is_private=False, max_members=200)
        if result:
            rooms.append(result)
        time.sleep(0.05)  # 极短延迟
    
    return rooms


def batch_login_users(base_url: str, user_count: int) -> List[Dict]:
    """批量登录用户"""
    print(f"\n登录 {user_count} 个测试用户...")
    
    users_info = []
    with ThreadPoolExecutor(max_workers=20) as executor:
        future_to_email = {}
        for i in range(1, user_count + 1):
            email = f"TestUser{i}@test.com"
            future = executor.submit(login_user, base_url, email, DEFAULT_PASSWORD)
            future_to_email[future] = email
        
        for future in as_completed(future_to_email):
            email = future_to_email[future]
            try:
                token = future.result()
                if token:
                    users_info.append({'email': email, 'token': token})
            except Exception:
                pass
    
    print(f"✅ 成功登录 {len(users_info)}/{user_count} 个用户")
    return users_info


def join_public_rooms(base_url: str, users_info: List[Dict], public_room_ids: List[str]):
    """用户加入公开房间"""
    print(f"\n用户加入 {len(public_room_ids)} 个公开房间...")
    
    def join_rooms_for_user(user: Dict) -> int:
        count = 0
        for room_id in public_room_ids:
            if join_room(base_url, user['token'], room_id):
                count += 1
        return count
    
    total_joins = 0
    with ThreadPoolExecutor(max_workers=30) as executor:
        future_to_user = {executor.submit(join_rooms_for_user, user): user for user in users_info}
        for future in as_completed(future_to_user):
            total_joins += future.result()
    
    print(f"✅ 用户加入房间完成，总加入次数: {total_joins}")


def simulate_mass_messages(ws_url: str, public_room_ids: List[str], users_info: List[Dict], message_count: int):
    """模拟大量消息发送（高并发版本）"""
    print("\n" + "=" * 60)
    print(f"开始发送 {message_count} 条消息（高并发模式）")
    print("=" * 60)
    
    # 为每个用户预生成消息列表
    user_messages = {}
    messages_per_user = message_count // len(users_info)
    
    for user in users_info:
        messages = []
        for _ in range(messages_per_user):
            template = random.choice(MESSAGE_TEMPLATES)
            if '{}' in template:
                content = template.format(random.randint(1, 999999))
            else:
                content = template
            messages.append(content)
        user_messages[user['email']] = {
            'token': user['token'],
            'messages': messages,
            'room_id': random.choice(public_room_ids)
        }
    
    # 使用线程池并发发送
    success_count = 0
    failed_count = 0
    start_time = time.time()
    
    def send_user_messages(email: str, data: Dict) -> int:
        return send_message_batch(ws_url, data['token'], data['room_id'], data['messages'])
    
    with ThreadPoolExecutor(max_workers=50) as executor:
        future_to_email = {
            executor.submit(send_user_messages, email, data): email 
            for email, data in user_messages.items()
        }
        
        completed = 0
        for future in as_completed(future_to_email):
            email = future_to_email[future]
            try:
                count = future.result()
                success_count += count
                completed += 1
                if completed % 10 == 0:
                    elapsed = time.time() - start_time
                    rate = success_count / elapsed if elapsed > 0 else 0
                    print(f"  进度: {completed}/{len(users_info)} 用户完成, "
                          f"已发送 {success_count} 条消息, "
                          f"速率: {rate:.1f} 条/秒")
            except Exception:
                failed_count += messages_per_user
    
    elapsed = time.time() - start_time
    rate = success_count / elapsed if elapsed > 0 else 0
    
    print(f"\n✅ 消息发送完成!")
    print(f"   成功: {success_count} 条")
    print(f"   失败: {failed_count} 条")
    print(f"   耗时: {elapsed:.2f} 秒")
    print(f"   速率: {rate:.1f} 条/秒")
    
    return success_count


def get_ws_url(port: int) -> str:
    """获取WebSocket URL"""
    return f"ws://localhost:{port}/ws"


def main():
    """主函数"""
    # 默认参数
    port = DEFAULT_PORT
    user_count = DEFAULT_USER_COUNT
    message_count = DEFAULT_MESSAGE_COUNT
    
    # 解析命令行参数
    if len(sys.argv) > 1:
        try:
            user_count = int(sys.argv[1])
        except ValueError:
            pass
    
    if len(sys.argv) > 2:
        try:
            message_count = int(sys.argv[2])
        except ValueError:
            pass
    
    if len(sys.argv) > 3:
        try:
            port = int(sys.argv[3])
        except ValueError:
            pass
    
    base_url = get_base_url(port)
    ws_url = get_ws_url(port)
    
    print("=" * 60)
    print("创建测试房间并模拟用户互发消息")
    print("=" * 60)
    print(f"服务器端口: {port}")
    print(f"测试用户数: {user_count}")
    print(f"目标消息数: {message_count}")
    print(f"基础URL: {base_url}")
    print(f"WebSocket: {ws_url}")
    print()
    
    # 检查服务器连接
    try:
        response = requests.get(f"{base_url}/api/health", timeout=5)
        print(f"✅ 服务器健康检查: 正常")
    except Exception as e:
        print(f"⚠️ 无法连接到服务器: {e}")
        print("请确保服务器已启动后再运行脚本")
        return
    
    # 1. 登录测试用户
    users_info = batch_login_users(base_url, user_count)
    
    if len(users_info) < 2:
        print("⚠️ 成功登录的用户数不足2个，无法继续测试")
        return
    
    # 2. 创建房间（只创建公开房间）
    creator_token = users_info[0]['token']
    rooms = create_test_rooms(base_url, creator_token, count=10)
    
    if not rooms:
        print("⚠️ 没有成功创建任何房间，无法继续测试")
        return
    
    # 筛选公开房间
    public_rooms = [(rid, is_private) for rid, is_private in rooms if not is_private]
    public_room_ids = [rid for rid, _ in public_rooms]
    
    print(f"\n✅ 成功创建 {len(rooms)} 个房间，其中公开房间 {len(public_room_ids)} 个")
    
    # 3. 用户加入公开房间
    join_public_rooms(base_url, users_info, public_room_ids)
    
    # 4. 模拟发送大量消息
    simulate_mass_messages(ws_url, public_room_ids, users_info, message_count)
    
    # 打印总结
    print("\n" + "=" * 60)
    print("测试完成总结")
    print("=" * 60)
    print(f"参与用户: {len(users_info)} 个")
    print(f"创建房间: {len(rooms)} 个（全部公开）")
    print(f"发送消息: {message_count} 条")
    print("\n房间列表:")
    for i, (room_id, is_private) in enumerate(rooms, 1):
        print(f"  {i}. {room_id} ({'私有' if is_private else '公开'})")
    print("\n现在可以在管理后台验证:")
    print("  1. 房间管理 - 查看房间列表和详情")
    print("  2. 消息管理 - 查看各房间的消息")
    print("=" * 60)


if __name__ == "__main__":
    main()
