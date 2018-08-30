# ChainX
##启动节点
###启动一个种子节点
```
RUST_LOG=debug ./Chainx validator
```
RUST_LOG=debug:输出所有打印信息
validator:出块
###启动一个同步节点
```
./Chainx bootnodes aaa --port bbb
```
aaa:指定其他节点P2P地址 如：/ip4/127.0.0.1/tcp/20222/p2p/QmZLZd1SFVhahpHACNfJqmGnXyg4yGJo1PJzUyWvd9ZFTE
bbb:指定本节点P2P的监听端口

##http API请求结构
###chain 相关rpc
1.chain_getHead (Get hash of the head.)

```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "chain_getHead", "id":123 }' 127.0.0.1:9933

返回： {"jsonrpc":"2.0","result":"0x448fa1631498bf813ac245e272c9e604a2ebf5da7e2979f05ada8c160bb88440","id":123}
```
2.chain_getHeader 获取中继链块的头部 (Get header of a relay chain block.)

```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "chain_getHeader", "params":["0x448fa1631498bf813ac245e272c9e604a2ebf5da7e2979f05ada8c160bb88440"],"id":123 }' 127.0.0.1:9933

返回：{"jsonrpc":"2.0","result":{"digest":{"logs":[]},"extrinsicsRoot":"0xc3c65ba2629d913f0c5ff4c41959b9dfca3e58c881e08cf3cbd6851b83ad7b7f","number":4264,"parentHash":"0x9b8b09959a3e1b88a6016676eefaf2bbf83a98bc9d9fa079aeba5c36fa296861","stateRoot":"0x77f5b0a4a8a7962a4fa7ec581ec9c227c9021b57395343346b75299a80e72718"},"id":123}
```
*pubsub chain_newHead*
1. subscribe_newHead
2. unsubscribe_newHead

###state 相关rpc
1. state_callAt
2. state_getStorageAt
3. state_getStorageHashAt
4. state_getStorageSizeAt
5. state_getStorageHash
6. state_getStorageSize
7. state_getStorage
8. state_call

###system 相关rpc
1.system_name

```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "system_name", "id":123 }' 127.0.0.1:9933

返回：{"jsonrpc":"2.0","result":"parity-polkadot","id":123}
```

2.system_version

```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "system_version", "id":123 }' 127.0.0.1:9933

返回：{"jsonrpc":"2.0","result":"0.2.0","id":123}
```
3.system_chain

```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "system_chain", "id":123 }' 127.0.0.1:9933

返回：{"jsonrpc":"2.0","result":"Krumme Lanke","id":123}
```
###author 相关rpc
1. author_submitRichExtrinsic
2. author_submitExtrinsic

*pubsub author_extrinsicUpdate*
1. author_submitAndWatchExtrinsic
2. author_unwatchExtrinsic
3. author_pendingExtrinsics 获取交易池中pending状态的交易列表
```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "author_pendingExtrinsics", "params":[], "id":123 }' 127.0.0.1:8081
```
```
返回：{"jsonrpc":"2.0","result":{},"id":123}
```
