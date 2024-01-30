import asyncio
import websockets

async def show_time(websocket):
    try:
        while True:
            await websocket.send(":3")
            await asyncio.sleep(60)
    except:
        pass

async def main():
    async with websockets.serve(show_time, "0.0.0.0", 4300):
        await asyncio.Future()  # run forever

if __name__ == "__main__":
    asyncio.run(main())
