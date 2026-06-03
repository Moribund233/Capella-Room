"""
PostgreSQL connection diagnostics for Capella Room.
Tests connectivity, service status, and common issues.
"""
import subprocess
import socket
import sys

DB_HOST = "localhost"
DB_PORT = 5432
DB_NAME = "capella_room_dev"
DB_USER = "developer"
DB_PASS = "dev123456"

def run(cmd, label=None):
    """Run a command and return output."""
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=10, shell=True)
        return r.stdout.strip() or r.stderr.strip()
    except Exception as e:
        return f"Error: {e}"

def section(title):
    print(f"\n{'='*60}")
    print(f" {title}")
    print(f"{'='*60}")

section("1. PostgreSQL 服务状态")

# Check Windows service
svc = run('sc query postgresql-x64-16 2>nul || sc query postgresql 2>nul', "PostgreSQL Service")
print(svc[:500] if svc else "(no Windows service found)")

# Check if pg_isready works
pg = run('pg_isready 2>nul || pg_isready -h localhost -p 5432 2>nul', "pg_isready")
print(f"pg_isready: {pg[:200] if pg else '(not available)'}")

# Check running processes
ps = run('tasklist /FI "IMAGENAME eq postgres.exe" 2>nul', "postgres.exe process")
if "postgres.exe" in ps:
    print("✅ PostgreSQL 进程运行中")
else:
    print("❌ PostgreSQL 进程未运行")

section("2. 端口连通性")

# Test TCP connection to port
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.settimeout(5)
result = sock.connect_ex((DB_HOST, DB_PORT))
sock.close()

if result == 0:
    print(f"✅ 端口 {DB_PORT} 开放并可连接")
else:
    print(f"❌ 无法连接到 {DB_HOST}:{DB_PORT} (错误码: {result})")
    print("  可能原因: PostgreSQL 未启动、端口未监听、或被防火墙阻止")

section("3. 网络连接测试")

# Use powershell to test TCP connection
ps_test = run(f'powershell -Command "Test-NetConnection -ComputerName {DB_HOST} -Port {DB_PORT} -WarningAction SilentlyContinue | Select-Object TcpTestSucceeded"')
print(ps_test)

section("4. psql 连接测试")

# Print connection string (password masked)
print(f"连接字符串: postgres://{DB_USER}:******@{DB_HOST}:{DB_PORT}/{DB_NAME}")

# Try psql connection
psql_test = run(f'set PGPASSWORD={DB_PASS}&& psql -h {DB_HOST} -p {DB_PORT} -U {DB_USER} -d {DB_NAME} -c "SELECT version();" 2>&1', "psql test")
if "version" in psql_test.lower() or "postgres" in psql_test.lower():
    print("✅ psql 连接成功!")
    for line in psql_test.split('\n')[:5]:
        print(f"   {line}")
else:
    print(f"❌ psql 连接失败:")
    for line in psql_test.split('\n')[:3]:
        print(f"   {line}")

    # Try just connecting without database
    psql_test2 = run(f'set PGPASSWORD={DB_PASS}&& psql -h {DB_HOST} -p {DB_PORT} -U {DB_USER} -d postgres -c "SELECT 1;" 2>&1', "psql no-db test")
    if "1" in psql_test2:
        print("✅ 到 postgres 默认数据库连接成功 (可能是指定数据库不存在)")
    else:
        print(f"❌ 到 postgres 默认数据库也失败:")
        for line in psql_test2.split('\n')[:2]:
            print(f"   {line}")

section("5. Python psycopg2 连接测试")

try:
    import psycopg2
    try:
        conn = psycopg2.connect(
            host=DB_HOST,
            port=DB_PORT,
            user=DB_USER,
            password=DB_PASS,
            dbname=DB_NAME,
            connect_timeout=5
        )
        cur = conn.cursor()
        cur.execute("SELECT version();")
        ver = cur.fetchone()
        print(f"✅ Python psycopg2 连接成功!")
        print(f"   PG Version: {ver[0][:80]}")
        cur.close()
        conn.close()
    except Exception as e:
        print(f"❌ psycopg2 连接失败: {e}")
except ImportError:
    print("(psycopg2 未安装, 跳过)")

    try:
        import pg8000
        try:
            conn = pg8000.connect(
                host=DB_HOST,
                port=DB_PORT,
                user=DB_USER,
                password=DB_PASS,
                database=DB_NAME,
                timeout=5
            )
            cur = conn.cursor()
            cur.execute("SELECT version();")
            ver = cur.fetchone()
            print(f"✅ Python pg8000 连接成功!")
            print(f"   PG Version: {ver[0][:80]}")
            cur.close()
            conn.close()
        except Exception as e:
            print(f"❌ pg8000 连接失败: {e}")
    except ImportError:
        print("(pg8000 也未安装, 跳过 Python 测试)")

section("6. 诊断结论")

# Check if the error log from the Rust server has more info
print("Rust 服务器报错: pool timed out while waiting for an open connection")
print()
print("可能的根本原因:")
print("  1. PostgreSQL 服务未启动 -> 需启动服务")
print("  2. PostgreSQL 监听端口不是 5432 -> 检查 pg_hba.conf 和 postgresql.conf")
print("  3. 密码认证失败 -> 检查 .env 文件中的凭证是否正确")
print("  4. 防火墙阻止连接 -> 检查 Windows 防火墙规则")
print("  5. 指定数据库 capella_room_dev 不存在 -> 需创建数据库")
print()
print("快速修复建议:")
print("  - 启动服务:  sc start postgresql-x64-16")
print("  - 创建数据库: createdb -U developer capella_room_dev")
print("  - 检查端口:   netstat -ano | findstr :5432")
