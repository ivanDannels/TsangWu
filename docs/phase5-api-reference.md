# 阶段5 API 快速参考

## Agent服务 API

### 基础路径
```
/api/agents
```

### 端点列表

#### 1. 创建Agent
```http
POST /agents
Content-Type: application/json

{
  "project_id": "uuid",
  "agent_type": "script" | "storyboard" | "visual" | "audio" | "quality",
  "config": {
    "agent_type": "script",
    "model": "qwen-7b",
    "temperature": 0.7,
    "max_tokens": 2000,
    "system_prompt": "你是一个专业的剧本创作助手"
  }
}

Response: Agent对象
```

#### 2. 获取Agent
```http
GET /agents/:id

Response: Agent对象
```

#### 3. 列出项目的Agent
```http
GET /agents/project/:project_id

Response: Agent[]
```

#### 4. 执行任务
```http
POST /agents/:id/execute
Content-Type: application/json

{
  "task_type": "generate_script",
  "input": {
    "theme": "唐朝宫廷",
    "style": "历史剧",
    "duration": 120
  }
}

Response: AgentTask对象
```

#### 5. 获取任务详情
```http
GET /agents/:id/tasks/:task_id

Response: AgentTask对象
```

## 文化引擎 API

### 基础路径
```
/api/culture
```

### 端点列表

#### 1. 创建朝代
```http
POST /dynasties
Content-Type: application/json

{
  "name": "唐朝",
  "period": "盛唐",
  "start_year": 618,
  "end_year": 907,
  "description": "中国历史上的黄金时代",
  "cultural_features": [
    "开放包容",
    "诗歌繁荣",
    "丝绸之路",
    "佛教兴盛"
  ]
}

Response: Dynasty对象
```

#### 2. 列出所有朝代
```http
GET /dynasties

Response: Dynasty[]
```

#### 3. 获取朝代详情
```http
GET /dynasties/:id

Response: Dynasty对象
```

#### 4. 创建文化元素
```http
POST /elements
Content-Type: application/json

{
  "dynasty_id": "uuid",
  "element_type": "architecture" | "costume" | "music" | "art" | "literature" | "custom",
  "name": "大明宫",
  "description": "唐朝皇宫，规模宏大",
  "tags": ["宫殿", "建筑", "皇家"],
  "reference_images": ["url1", "url2"],
  "metadata": {
    "location": "长安",
    "built_year": 634
  }
}

Response: CultureElement对象
```

#### 5. 获取元素详情
```http
GET /elements/:id

Response: CultureElement对象
```

#### 6. 列出朝代的文化元素
```http
GET /dynasties/:id/elements?element_type=architecture

Response: CultureElement[]
```

#### 7. 验证内容
```http
POST /validate
Content-Type: application/json

{
  "dynasty_id": "uuid",
  "content_type": "script",
  "content": {
    "text": "剧本内容...",
    "scenes": [...]
  }
}

Response: {
  "valid": true,
  "score": 0.92,
  "issues": [
    {
      "severity": "warning",
      "message": "建议添加更多唐朝特色元素",
      "field": "scene_1",
      "suggestion": "可以加入大明宫、长安街市等场景"
    }
  ],
  "suggestions": [
    "参考唐朝时期的典型文化元素",
    "确保服饰、建筑等细节符合历史考证"
  ]
}
```

#### 8. 推荐文化元素
```http
POST /recommend
Content-Type: application/json

{
  "dynasty_id": "uuid",
  "element_type": "costume",
  "context": "宫廷宴会场景",
  "limit": 10
}

Response: {
  "elements": CultureElement[],
  "reasoning": "基于唐朝朝代的costume类型元素，推荐了10个相关文化元素"
}
```

#### 9. 获取知识图谱
```http
GET /dynasties/:id/knowledge-graph

Response: {
  "nodes": [
    {
      "id": "uuid",
      "node_type": "dynasty",
      "name": "唐朝",
      "properties": {...}
    },
    {
      "id": "uuid",
      "node_type": "architecture",
      "name": "大明宫",
      "properties": {...}
    }
  ],
  "relations": [
    {
      "from_id": "dynasty_uuid",
      "to_id": "element_uuid",
      "relation_type": "has_element",
      "properties": {}
    }
  ]
}
```

## 数据模型

### Agent
```typescript
{
  id: UUID,
  project_id: UUID,
  agent_type: "script" | "storyboard" | "visual" | "audio" | "quality",
  status: "idle" | "running" | "paused" | "error",
  config: AgentConfig,
  created_at: DateTime,
  updated_at: DateTime
}
```

### AgentTask
```typescript
{
  id: UUID,
  agent_id: UUID,
  task_type: string,
  input: JSON,
  output: JSON | null,
  status: string,
  error: string | null,
  created_at: DateTime,
  completed_at: DateTime | null
}
```

### Dynasty
```typescript
{
  id: UUID,
  name: string,
  period: string,
  start_year: number,
  end_year: number,
  description: string,
  cultural_features: string[],
  created_at: DateTime
}
```

### CultureElement
```typescript
{
  id: UUID,
  dynasty_id: UUID,
  element_type: "architecture" | "costume" | "music" | "art" | "literature" | "custom",
  name: string,
  description: string,
  tags: string[],
  reference_images: string[],
  metadata: JSON,
  created_at: DateTime
}
```

## 错误响应格式

所有API错误都返回统一格式：

```json
{
  "error": "错误描述信息"
}
```

### HTTP状态码
- `200 OK`: 成功
- `400 Bad Request`: 请求参数错误
- `404 Not Found`: 资源不存在
- `409 Conflict`: 资源冲突（如Agent忙碌）
- `500 Internal Server Error`: 服务器内部错误

## 使用流程示例

### 1. 创建剧本生成工作流

```bash
# 1. 创建Script Agent
AGENT_ID=$(curl -X POST http://localhost:8080/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "project_id": "'$PROJECT_ID'",
    "agent_type": "script",
    "config": {
      "agent_type": "script",
      "model": "qwen-7b"
    }
  }' | jq -r '.id')

# 2. 执行剧本生成任务
TASK_ID=$(curl -X POST http://localhost:8080/api/agents/$AGENT_ID/execute \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "generate_script",
    "input": {
      "theme": "唐朝宫廷",
      "style": "历史剧"
    }
  }' | jq -r '.id')

# 3. 查询任务状态
curl http://localhost:8080/api/agents/$AGENT_ID/tasks/$TASK_ID
```

### 2. 文化验证工作流

```bash
# 1. 获取唐朝信息
DYNASTY_ID=$(curl http://localhost:8080/api/culture/dynasties \
  | jq -r '.[] | select(.name=="唐朝") | .id')

# 2. 验证剧本内容
curl -X POST http://localhost:8080/api/culture/validate \
  -H "Content-Type: application/json" \
  -d '{
    "dynasty_id": "'$DYNASTY_ID'",
    "content_type": "script",
    "content": {
      "text": "剧本内容..."
    }
  }'

# 3. 获取推荐的文化元素
curl -X POST http://localhost:8080/api/culture/recommend \
  -H "Content-Type: application/json" \
  -d '{
    "dynasty_id": "'$DYNASTY_ID'",
    "element_type": "costume",
    "limit": 5
  }'
```

## 注意事项

1. **认证**: 生产环境需要添加JWT认证
2. **限流**: 建议对API添加速率限制
3. **异步任务**: Agent任务是异步执行的，需要轮询或使用WebSocket获取结果
4. **数据库**: 确保数据库迁移已执行
5. **PyBridge**: 当前为模拟实现，生产环境需要连接真实的AI服务
