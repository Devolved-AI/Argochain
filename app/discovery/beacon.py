
import asyncio
from aiokademlia.network import Server

class Beacon:
    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port
        self.server = Server()

    async def start(self):
        await self.server.listen(self.port, interface=self.host)
        bootstrap_node = ('127.0.0.1', 8468)
        await self.server.bootstrap([bootstrap_node])
        print(f"Beacon node started at {self.host}:{self.port}")

    async def stop(self):
        await self.server.stop()

    async def set(self, key, value):
        await self.server.set(key, value)

    async def get(self, key):
        result = await self.server.get(key)
        return result
