---
name: convex-db
description: "Integrate Convex database into web apps with best practices for real-time sync, type-safe schemas, and reactive queries. Covers schema design, function patterns (queries/mutations/actions), React/Next.js integration, authentication with Clerk, file storage, and scheduling. Trigger when users build with Convex or ask about Convex database patterns, gotchas, or integration."
---

# Convex DB Integration

Build real-time applications with Convex—the reactive database that syncs automatically.

## Philosophy: Reactive by Design, Not by Accident

Convex is fundamentally different from traditional databases. It's a **reactive system** where queries track dependencies and automatically push updates to clients. This changes everything about how you think about data access.

**Before writing Convex code, understand these mental models**:

1. **Queries are Subscriptions**: Not one-time fetches—live connections that update automatically when data changes
2. **Mutations are Transactions**: Atomic, serializable, with automatic rollback on failure
3. **Actions for Side Effects**: Keep mutations pure; delegate external I/O (APIs, LLMs, emails) to actions
4. **Indexes First**: Performance is explicit—no query planner surprises, you control exactly what index is used
5. **Type Safety End-to-End**: From schema definition to React hooks, TypeScript ensures correctness

**Key insight**: Every document your query reads creates a dependency. When any dependency changes, the query re-runs and clients receive updates. This is powerful but has implications for performance.

---

## Core Concepts

### The Three Function Types

| Type | Purpose | Database Access | Side Effects | Reactive |
|------|---------|-----------------|--------------|----------|
| **Query** | Read data | Read-only | None allowed | Yes - auto-updates |
| **Mutation** | Write data | Read/Write | None allowed | Transaction - all or nothing |
| **Action** | External I/O | Via queries/mutations | Allowed | One-shot, not reactive |

```typescript
// Query: Subscribe to real-time data
export const listMessages = query({
  args: { channelId: v.id("channels") },
  returns: v.array(v.object({ /* ... */ })),
  handler: async (ctx, args) => {
    return await ctx.db
      .query("messages")
      .withIndex("by_channel", (q) => q.eq("channelId", args.channelId))
      .order("desc")
      .take(50);
  },
});

// Mutation: Transactional write
export const sendMessage = mutation({
  args: { channelId: v.id("channels"), text: v.string() },
  returns: v.id("messages"),
  handler: async (ctx, args) => {
    const identity = await ctx.auth.getUserIdentity();
    if (!identity) throw new Error("Not authenticated");

    return await ctx.db.insert("messages", {
      channelId: args.channelId,
      text: args.text,
      authorId: identity.subject,
    });
  },
});

// Action: External API calls
export const generateAIResponse = action({
  args: { prompt: v.string() },
  returns: v.string(),
  handler: async (ctx, args) => {
    const response = await openai.chat.completions.create({ /* ... */ });
    return response.choices[0].message.content;
  },
});
```

### Function Visibility

```typescript
// Public: Callable from client
import { query, mutation, action } from "./_generated/server";

// Internal: Only callable from other functions
import { internalQuery, internalMutation, internalAction } from "./_generated/server";
```

**Rule**: Use `internal*` for sensitive operations. Public functions are your API surface.

---

## Schema Design

### Always Define a Schema

```typescript
// convex/schema.ts
import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  users: defineTable({
    name: v.string(),
    email: v.string(),
    avatarUrl: v.optional(v.string()),
    role: v.union(v.literal("admin"), v.literal("member")),
  })
    .index("by_email", ["email"]),

  messages: defineTable({
    channelId: v.id("channels"),
    authorId: v.id("users"),
    text: v.string(),
    editedAt: v.optional(v.number()),
  })
    .index("by_channel", ["channelId"])
    .index("by_author", ["authorId"]),

  channels: defineTable({
    name: v.string(),
    isPrivate: v.boolean(),
  }),
});
```

### Validator Types

| Validator | TypeScript | Notes |
|-----------|------------|-------|
| `v.string()` | `string` | |
| `v.number()` | `number` | Float64 |
| `v.int64()` | `bigint` | For large integers |
| `v.boolean()` | `boolean` | |
| `v.null()` | `null` | **Not undefined** |
| `v.id("table")` | `Id<"table">` | Document reference |
| `v.array(v.X)` | `X[]` | |
| `v.object({})` | `{ field: type }` | |
| `v.optional(v.X)` | `X \| undefined` | Omittable field |
| `v.union(a, b)` | `A \| B` | Discriminated unions |
| `v.literal("x")` | `"x"` | Exact value |

**Critical**: `v.map()` and `v.set()` are NOT supported. Use arrays or objects.

**Critical**: JavaScript `undefined` is not a valid Convex value. Use `null` instead.

### Index Strategy

Indexes are **essential** for performance. Without them, queries scan entire tables.

```typescript
// Define indexes for your query patterns
defineTable({
  channelId: v.id("channels"),
  authorId: v.id("users"),
  createdAt: v.number(),
})
  .index("by_channel", ["channelId"])                    // Filter by channel
  .index("by_channel_and_time", ["channelId", "createdAt"]) // Filter + order
  .index("by_author", ["authorId"])                      // Filter by author
```

**Index rules**:
- Indexes like `by_foo` and `by_foo_and_bar` are redundant—keep only `by_foo_and_bar`
- Order index fields by: equality filters first, then range/sort fields
- Each additional index adds write overhead—don't over-index

See references/schema-patterns.md for relationship patterns and advanced schema design.

---

## Query Patterns

### Use Indexes, Not Filter

```typescript
// BAD: Scans entire table, then filters
const messages = await ctx.db
  .query("messages")
  .filter((q) => q.eq(q.field("channelId"), args.channelId))
  .collect();

// GOOD: Uses index, efficient
const messages = await ctx.db
  .query("messages")
  .withIndex("by_channel", (q) => q.eq("channelId", args.channelId))
  .collect();
```

### Pagination for Large Results

```typescript
export const listMessages = query({
  args: {
    channelId: v.id("channels"),
    paginationOpts: paginationOptsValidator,
  },
  returns: v.object({
    page: v.array(/* message schema */),
    isDone: v.boolean(),
    continueCursor: v.string(),
  }),
  handler: async (ctx, args) => {
    return await ctx.db
      .query("messages")
      .withIndex("by_channel", (q) => q.eq("channelId", args.channelId))
      .order("desc")
      .paginate(args.paginationOpts);
  },
});
```

### Single Document Retrieval

```typescript
// By ID (most common)
const user = await ctx.db.get(args.userId);

// Unique by index (throws if multiple match)
const user = await ctx.db
  .query("users")
  .withIndex("by_email", (q) => q.eq("email", args.email))
  .unique();

// First match
const latestMessage = await ctx.db
  .query("messages")
  .withIndex("by_channel", (q) => q.eq("channelId", args.channelId))
  .order("desc")
  .first();
```

See references/function-patterns.md for helper functions and advanced patterns.

---

## Mutation Patterns

### Basic CRUD

```typescript
// Create
const id = await ctx.db.insert("users", {
  name: args.name,
  email: args.email,
  role: "member",
});

// Update (merge)
await ctx.db.patch(args.userId, {
  name: args.newName,
});

// Replace (full document)
await ctx.db.replace(args.userId, {
  name: args.name,
  email: args.email,
  role: args.role,
});

// Delete
await ctx.db.delete(args.userId);
```

### Authentication Check Pattern

```typescript
export const updateProfile = mutation({
  args: { name: v.string() },
  returns: v.null(),
  handler: async (ctx, args) => {
    const identity = await ctx.auth.getUserIdentity();
    if (!identity) {
      throw new Error("Not authenticated");
    }

    const user = await ctx.db
      .query("users")
      .withIndex("by_clerk_id", (q) => q.eq("clerkId", identity.subject))
      .unique();

    if (!user) {
      throw new Error("User not found");
    }

    await ctx.db.patch(user._id, { name: args.name });
    return null;
  },
});
```

---

## Action Patterns

### When to Use Actions

Use actions for:
- External API calls (OpenAI, Stripe, SendGrid)
- File operations beyond Convex storage
- Operations that might fail and shouldn't block transactions

### Action → Mutation Pattern

Actions can't write to the database directly. Schedule a mutation:

```typescript
export const processPayment = action({
  args: { orderId: v.id("orders"), paymentMethodId: v.string() },
  returns: v.null(),
  handler: async (ctx, args) => {
    // Call external API
    const payment = await stripe.paymentIntents.create({ /* ... */ });

    // Write result back via mutation
    await ctx.runMutation(internal.orders.markPaid, {
      orderId: args.orderId,
      stripePaymentId: payment.id,
    });

    return null;
  },
});
```

### Mutation + Scheduled Action Pattern

For reliability, start from a mutation and schedule the action:

```typescript
export const sendEmail = mutation({
  args: { userId: v.id("users"), template: v.string() },
  returns: v.null(),
  handler: async (ctx, args) => {
    // Record intent in database
    const emailId = await ctx.db.insert("pendingEmails", {
      userId: args.userId,
      template: args.template,
      status: "pending",
    });

    // Schedule the action (runs after mutation commits)
    await ctx.scheduler.runAfter(0, internal.emails.send, { emailId });

    return null;
  },
});
```

---

## React Integration

### Provider Setup

```tsx
// main.tsx or _app.tsx
import { ConvexProvider, ConvexReactClient } from "convex/react";

const convex = new ConvexReactClient(import.meta.env.VITE_CONVEX_URL);

function App() {
  return (
    <ConvexProvider client={convex}>
      <YourApp />
    </ConvexProvider>
  );
}
```

### Query Hooks

```tsx
import { useQuery } from "convex/react";
import { api } from "../convex/_generated/api";

function MessageList({ channelId }: { channelId: Id<"channels"> }) {
  const messages = useQuery(api.messages.list, { channelId });

  // messages is undefined while loading, then array
  if (messages === undefined) return <Loading />;

  return (
    <ul>
      {messages.map((msg) => (
        <li key={msg._id}>{msg.text}</li>
      ))}
    </ul>
  );
}
```

### Mutation Hooks

```tsx
import { useMutation } from "convex/react";
import { api } from "../convex/_generated/api";

function SendMessage({ channelId }: { channelId: Id<"channels"> }) {
  const sendMessage = useMutation(api.messages.send);
  const [text, setText] = useState("");

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    await sendMessage({ channelId, text });
    setText("");
  };

  return (
    <form onSubmit={handleSubmit}>
      <input value={text} onChange={(e) => setText(e.target.value)} />
      <button type="submit">Send</button>
    </form>
  );
}
```

### Paginated Queries

```tsx
import { usePaginatedQuery } from "convex/react";

function InfiniteMessages({ channelId }: { channelId: Id<"channels"> }) {
  const { results, status, loadMore } = usePaginatedQuery(
    api.messages.list,
    { channelId },
    { initialNumItems: 20 }
  );

  return (
    <>
      {results.map((msg) => <Message key={msg._id} message={msg} />)}
      {status === "CanLoadMore" && (
        <button onClick={() => loadMore(20)}>Load more</button>
      )}
    </>
  );
}
```

See references/integration-patterns.md for Next.js App Router, authentication, and advanced patterns.

---

## Anti-Patterns to Avoid

### 1. Unbounded `.collect()`

```typescript
// DANGEROUS: Loads entire table
const allUsers = await ctx.db.query("users").collect();

// SAFE: Limit results
const users = await ctx.db.query("users").take(100);
// Or use pagination
const users = await ctx.db.query("users").paginate(opts);
```

### 2. Filter Without Index

```typescript
// SLOW: Scans table, filters in memory
await ctx.db.query("messages")
  .filter((q) => q.eq(q.field("authorId"), args.authorId))
  .collect();

// FAST: Uses index
await ctx.db.query("messages")
  .withIndex("by_author", (q) => q.eq("authorId", args.authorId))
  .collect();
```

### 3. External Calls in Mutations

```typescript
// WRONG: Mutations can't have side effects
export const sendEmail = mutation({
  handler: async (ctx, args) => {
    await sendgrid.send(/* ... */); // NO! This breaks transactionality
  },
});

// RIGHT: Use an action
export const sendEmail = action({
  handler: async (ctx, args) => {
    await sendgrid.send(/* ... */); // Actions can have side effects
  },
});
```

### 4. Using `ctx.runQuery` Instead of Helper Functions

```typescript
// UNNECESSARY: Creates overhead
export const getUser = query({
  handler: async (ctx, args) => {
    const user = await ctx.runQuery(api.users.getById, { id: args.userId });
    // ...
  },
});

// BETTER: Use a plain TypeScript function
async function getUser(ctx: QueryCtx, userId: Id<"users">) {
  return await ctx.db.get(userId);
}

export const getProfile = query({
  handler: async (ctx, args) => {
    const user = await getUser(ctx, args.userId);
    // ...
  },
});
```

### 5. Queries That Read Frequently-Updating Documents

```typescript
// PROBLEMATIC: Re-runs every time any counter changes
export const getStats = query({
  handler: async (ctx) => {
    const counters = await ctx.db.query("counters").collect();
    return counters.reduce((sum, c) => sum + c.value, 0);
  },
});

// BETTER: Denormalize or use a summary table updated less frequently
```

### 6. Using `undefined` Instead of `null`

```typescript
// WRONG: undefined is not a valid Convex value
return { user: undefined }; // Will become null anyway

// RIGHT: Use null explicitly
return { user: null };
```

See references/gotchas.md for more common mistakes and solutions.

---

## Scheduling

### One-Time Scheduled Functions

```typescript
// Schedule for later
await ctx.scheduler.runAfter(
  1000 * 60 * 5, // 5 minutes
  internal.notifications.sendReminder,
  { userId: args.userId }
);

// Schedule for specific time
await ctx.scheduler.runAt(
  new Date("2024-01-01T00:00:00Z").getTime(),
  internal.events.processNewYear,
  {}
);
```

### Cron Jobs

```typescript
// convex/crons.ts
import { cronJobs } from "convex/server";
import { internal } from "./_generated/api";

const crons = cronJobs();

// Every hour
crons.interval("cleanup", { hours: 1 }, internal.maintenance.cleanup, {});

// Daily at midnight UTC
crons.daily("reports", { hourUTC: 0, minuteUTC: 0 }, internal.reports.generate, {});

// Cron syntax
crons.cron("backup", "0 */6 * * *", internal.backup.run, {}); // Every 6 hours

export default crons;
```

---

## File Storage

### Upload Files

```typescript
// Generate upload URL (in mutation)
export const generateUploadUrl = mutation({
  returns: v.string(),
  handler: async (ctx) => {
    return await ctx.storage.generateUploadUrl();
  },
});

// Client-side upload
const uploadUrl = await generateUploadUrl();
const result = await fetch(uploadUrl, {
  method: "POST",
  headers: { "Content-Type": file.type },
  body: file,
});
const { storageId } = await result.json();
```

### Get File URL

```typescript
export const getImageUrl = query({
  args: { storageId: v.id("_storage") },
  returns: v.union(v.string(), v.null()),
  handler: async (ctx, args) => {
    return await ctx.storage.getUrl(args.storageId);
  },
});
```

### File Metadata

```typescript
// Query the _storage system table
const metadata = await ctx.db.system.get(args.storageId);
// Returns: { contentType, sha256, size }
```

---

## Variation Guidance

**Project structure should vary based on complexity**:

- **Simple apps**: All functions in `convex/` root
- **Medium apps**: Group by domain (`convex/users/`, `convex/messages/`)
- **Complex apps**: Domain folders + shared validators + helper modules

**Authentication approach should match your stack**:

- Next.js + Clerk → Use `ConvexProviderWithClerk`
- Custom auth → Implement JWT validation
- No auth needed → Skip auth layer

**Real-time patterns should match your needs**:

- Full real-time → Use `useQuery` everywhere
- Hybrid → `useQuery` for live data, `fetchQuery` for static
- Performance-critical → Consider query granularity carefully

---

## Remember

**Convex is a reactive database—embrace it, don't fight it.**

The best Convex apps:
- Let reactivity flow naturally through queries
- Keep mutations pure and atomic
- Delegate side effects to actions
- Use indexes explicitly for predictable performance
- Leverage TypeScript for end-to-end safety

Convex eliminates the database layer complexity—no ORMs, no connection pools, no cache invalidation. In return, think carefully about:
- What data each query reads (reactivity implications)
- Index design (performance is explicit)
- Function boundaries (query vs mutation vs action)

**Claude is capable of building sophisticated real-time applications with Convex. These guidelines illuminate proven patterns—they don't limit what's possible.**
