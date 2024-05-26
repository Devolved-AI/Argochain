
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from app.discovery.beacon import Beacon
from app.discovery.node import DiscoveryNode
import asyncio

app = FastAPI()

# Create and start the beacon and discovery nodes
beacon = Beacon('127.0.0.1', 8468)
discovery_node = DiscoveryNode('127.0.0.1', 8470)

class KeyValue(BaseModel):
    key: str
    value: str

@app.on_event("startup")
async def startup_event():
    asyncio.create_task(beacon.start())
    asyncio.create_task(discovery_node.start())

@app.on_event("shutdown")
async def shutdown_event():
    await beacon.stop()
    await discovery_node.stop()

@app.post("/set/")
async def set_value(kv: KeyValue):
    await discovery_node.set(kv.key, kv.value.encode())
    return {"message": "Value set successfully"}

@app.get("/get/{key}")
async def get_value(key: str):
    value = await discovery_node.get(key)
    if value is None:
        raise HTTPException(status_code=404, detail="Key not found")
    return {"key": key, "value": value.decode()}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
