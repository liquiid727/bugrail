// T08 fixture mock LLM: OpenAI-compatible /v1/chat/completions endpoint that
// returns a deterministic L1 extraction. Memory content is derived from
// tokens embedded in the captured conversation text so per-team isolation
// stays observable. No real credentials; offline fixture only.
import http from "node:http"

const server = http.createServer((req, res) => {
  let body = ""
  req.on("data", (c) => (body += c))
  req.on("end", () => {
    if (req.url !== "/v1/chat/completions") {
      res.writeHead(404).end()
      return
    }
    let token = "generic"
    const match = body.match(/T8TOK-([a-z0-9-]+)/)
    if (match) token = match[1]
    const content = JSON.stringify([
      {
        scene_name: "bugrail-t08-scene",
        message_ids: [],
        memories: [
          {
            content: `memory-of:${token} captured fact for later recall`,
            type: "episodic",
            priority: 50,
            source_message_ids: [],
            metadata: {},
          },
        ],
      },
    ])
    res.writeHead(200, { "content-type": "application/json" })
    res.end(
      JSON.stringify({
        id: "chatcmpl-t08",
        object: "chat.completion",
        created: Math.floor(Date.now() / 1000),
        model: "mock-t08",
        choices: [{ index: 0, message: { role: "assistant", content }, finish_reason: "stop" }],
        usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 },
      })
    )
  })
})

server.listen(18100, "127.0.0.1", () => console.log("mock-llm on 127.0.0.1:18100"))
