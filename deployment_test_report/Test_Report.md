# Load Testing with JMeter: Report for WebSocket Connection

### Introduction
Load testing is a critical aspect of performance testing, aimed at evaluating how a system behaves under high load conditions. For this report, we used Apache JMeter, a popular open-source testing tool, to conduct a load test on a WebSocket endpoint wss://explorer.devolvedai.com. This endpoint represents a connection to a blockchain-based service, allowing real-time communication and data transfer.

### Objective
The primary objective of the load test was to determine the performance characteristics of the WebSocket connection under varying load conditions. This includes assessing metrics such as response time, error rate, throughput, and resource utilization. By simulating multiple concurrent users and interactions, the test aimed to identify potential bottlenecks, scalability issues, and overall stability.

### Test Configuration
The load test was configured with the following parameters:

- WebSocket Endpoint: wss://explorer.devolvedai.com

- Load Generator: Apache JMeter

- Concurrency Levels: The test was run with varying numbers of concurrent users to simulate different load scenarios. This thread ranged from 1,0000 users per second.

- Test Duration: Each test run lasted for 10 minutes to collect sufficient data for analysis.

- Test Scenarios: The test included scenarios such as establishing a WebSocket connection, sending messages, and receiving responses.


![image (1)](https://github.com/Devolved-AI/Argochain/assets/160380027/a7b6b5ff-dfa9-4708-be0a-3f6acbe39af1)



