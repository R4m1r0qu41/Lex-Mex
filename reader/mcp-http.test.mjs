import test from 'node:test';import assert from 'node:assert/strict';import {Client} from '@modelcontextprotocol/sdk/client/index.js';import {StreamableHTTPClientTransport} from '@modelcontextprotocol/sdk/client/streamableHttp.js';import {handleMcp} from './mcp-http.mjs';import {loadFixture} from './query-fixture.mjs';
test('official SDK client initializes, discovers tools and retrieves release-pinned evidence',async()=>{
 const adapter=await loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url));const client=new Client({name:'lex-test',version:'1'});
 const transport=new StreamableHTTPClientTransport(new URL('https://example.test/lex-mex/mcp'),{fetch:async(url,init)=>handleMcp(new Request(url,init),adapter)});
 try{await client.connect(transport);assert.equal((await client.listTools()).tools.length,4);const result=await client.callTool({name:'get_provision',arguments:{release_id:adapter.release.id,instrument:'norma-demo',type:'article',number:'1'}});assert.equal(result.isError,false);assert.equal(result.structuredContent.release.id,adapter.release.id);}finally{await client.close();}
 assert.equal((await handleMcp(new Request('https://example.test/lex-mex/mcp',{headers:{origin:'https://evil.test'}}),adapter)).status,403);
});

test('oversized MCP requests are rejected before protocol execution',async()=>{
 const adapter=await loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url));const response=await handleMcp(new Request('https://example.test/lex-mex/mcp',{method:'POST',body:'x'.repeat(8193)}),adapter);assert.equal(response.status,413);
});
