# -*- coding: utf-8 -*-

import asyncio
import websockets
import json
import time
import hashlib

WS_URL = ""  # Change "spot" to "futures" when interacting with WS ports
access_id = ""  # Replace with your access id
secret_key = ""  # Replace with your secret key


async def ping(conn):
    param = {"method": "server.ping", "params": {}, "id": 1}
    while True:
        await conn.send(json.dumps(param))
        await asyncio.sleep(3)


async def auth(conn):
    timestamp = int(time.time() * 1000)

    # Generate your signature string
    prepared_str = f"{timestamp}{secret_key}"
    signed_str = hashlib.sha256(prepared_str.encode("utf-8")).hexdigest().lower()

    param = {
        "method": "server.sign",
        "params": {
            "access_id": access_id,
            "signed_str": signed_str,
            "timestamp": timestamp,
        },
        "id": 1,
    }
    await conn.send(json.dumps(param))
    res = await conn.recv()
    print("Authentication Result: ", json.loads(res))


async def subscribe_depth(conn):
    param = {
        "method": "depth.subscribe",
        "params": {"market_list": [["BTCUSDT", 5, "0", True]]},
        "id": 1,
    }
    await conn.send(json.dumps(param))
    res = await conn.recv()
    print(json.loads(res))


async def subscribe_asset(conn):
    param = {"method": "balance.subscribe", "params": {"ccy_list": ["USDT"]}, "id": 1}
    await conn.send(json.dumps(param))
    res = await conn.recv()
    print(json.loads(res))


async def main():
    try:
        # Note: Must close websockets ping feature before creating a new ping task, set ping_interval to None
        async with websockets.connect(
            uri=WS_URL, compression=None, ping_interval=None
        ) as conn:
            await auth(conn)
            await subscribe_depth(conn)
            await subscribe_asset(conn)

            asyncio.create_task(ping(conn))

            while True:
                res = await conn.recv()
                res = json.loads(res)
                print(res)
    except Exception as e:
        print(f"An error occurred: {e}")


if __name__ == "__main__":
    asyncio.run(main())
