# Common Gotchas and Solutions

Mistakes developers commonly make when building with Convex, and how to avoid them.

## Database Operations

### Unbounded `.collect()` Kills Performance

**Problem**: Loading an entire table into memory.

```typescript
// DANGEROUS: Loads ALL users
const users = await ctx.db.query("users").collect();

// Could be thousands of documents, causing:
// - Memory exhaustion
// - Slow responses
// - Transaction limits exceeded
```

**Solution**: Always limit results.

```typescript
// Use take() for fixed limits
const users = await ctx.db.query("users").take(100);

// Use pagination for user-controlled loading
const users = await ctx.db.query("users").paginate(opts);

// Use an index to narrow before collecting
const activeUsers = await ctx.db
  .query("users")
  .withIndex("by_status", (q) => q.eq("status", "active"))
  .take(100);
```

---

### `.filter()` Scans the Entire Table

**Problem**: Filter happens after loading documents.

```typescript
// SLOW: Loads all messages, then filters
const messages = await ctx.db
  .query("messages")
  .filter((q) => q.eq(q.field("channelId"), channelId))
  .collect();
```

**Solution**: Use indexes for filtering.

```typescript
// FAST: Only loads matching documents
const messages = await ctx.db
  .query("messages")
  .withIndex("by_channel", (q) => q.eq("channelId", channelId))
  .collect();
```

**Rule**: Search your codebase for `.filter(` and evaluate each one. If filtering over 1000+ documents, add an index.

---

### Redundant Indexes Waste Storage

**Problem**: Creating overlapping indexes.

```typescript
// REDUNDANT: by_user is a subset of by_user_and_channel
.index("by_user", ["userId"])
.index("by_user_and_channel", ["userId", "channelId"])
```

**Solution**: Only keep the compound index.

```typescript
// This single index supports both query patterns:
.index("by_user_and_channel", ["userId", "channelId"])

// Query by user only:
.withIndex("by_user_and_channel", (q) => q.eq("userId", userId))

// Query by user AND channel:
.withIndex("by_user_and_channel", (q) =>
  q.eq("userId", userId).eq("channelId", channelId)
)
```

---

### Using `.delete()` on Query Results

**Problem**: Trying to delete while iterating.

```typescript
// ERROR: Can't call delete on query
await ctx.db
  .query("messages")
  .filter(q => q.lt(q.field("createdAt"), cutoffTime))
  .delete();  // This doesn't exist!
```

**Solution**: Collect first, then delete each.

```typescript
const oldMessages = await ctx.db
  .query("messages")
  .withIndex("by_time", (q) => q.lt("createdAt", cutoffTime))
  .collect();

await Promise.all(oldMessages.map((m) => ctx.db.delete(m._id)));
```

---

## Function Design

### External Calls in Mutations

**Problem**: Mutations must be deterministic.

```typescript
// WRONG: Mutations can't have side effects
export const processOrder = mutation({
  handler: async (ctx, args) => {
    // This breaks transactionality!
    const payment = await stripe.charges.create({ /* ... */ });

    await ctx.db.insert("orders", {
      paymentId: payment.id,
    });
  },
});
```

**Solution**: Use actions for external calls.

```typescript
// RIGHT: Action for external call
export const processOrder = action({
  handler: async (ctx, args) => {
    const payment = await stripe.charges.create({ /* ... */ });

    // Write result via mutation
    await ctx.runMutation(internal.orders.create, {
      paymentId: payment.id,
    });
  },
});

// Or: Start from mutation, schedule action
export const initiateOrder = mutation({
  handler: async (ctx, args) => {
    const orderId = await ctx.db.insert("orders", { status: "pending" });

    await ctx.scheduler.runAfter(0, internal.orders.processPayment, {
      orderId,
    });

    return orderId;
  },
});
```

---

### Using `ctx.runQuery` Instead of Helper Functions

**Problem**: Unnecessary overhead for internal reuse.

```typescript
// INEFFICIENT: Extra function call overhead
export const getUser = query({
  handler: async (ctx, args) => {
    return await ctx.db.get(args.userId);
  },
});

export const getUserWithPosts = query({
  handler: async (ctx, args) => {
    // Why run a separate query when we have ctx?
    const user = await ctx.runQuery(api.users.getUser, { userId: args.userId });
    const posts = await ctx.db.query("posts")...
  },
});
```

**Solution**: Use plain TypeScript functions.

```typescript
// BETTER: Direct helper function
async function getUser(ctx: QueryCtx, userId: Id<"users">) {
  return await ctx.db.get(userId);
}

export const getUserWithPosts = query({
  handler: async (ctx, args) => {
    const user = await getUser(ctx, args.userId);  // Direct call
    const posts = await ctx.db.query("posts")...
  },
});
```

**Use `ctx.runQuery`/`ctx.runMutation` only for**:
- Calling component functions
- When you need partial rollback semantics

---

### Forgetting Argument Validators

**Problem**: No runtime validation, types only at compile time.

```typescript
// WEAK: No runtime validation
export const createUser = mutation({
  handler: async (ctx, args: { name: string; email: string }) => {
    // A malicious client could send anything
  },
});
```

**Solution**: Always use validators.

```typescript
// STRONG: Runtime validation
export const createUser = mutation({
  args: {
    name: v.string(),
    email: v.string(),
  },
  returns: v.id("users"),
  handler: async (ctx, args) => {
    // args is guaranteed to match schema
  },
});
```

---

## Data Types

### Using `undefined` Instead of `null`

**Problem**: JavaScript `undefined` is not a valid Convex value.

```typescript
// WRONG: undefined becomes null anyway
export const getUser = query({
  handler: async (ctx, args) => {
    const user = await ctx.db.get(args.userId);
    if (!user) return undefined;  // Will become null
    return user;
  },
});
```

**Solution**: Use `null` explicitly.

```typescript
// RIGHT: Explicit null
export const getUser = query({
  args: { userId: v.id("users") },
  returns: v.union(v.object({ /* user schema */ }), v.null()),
  handler: async (ctx, args) => {
    const user = await ctx.db.get(args.userId);
    if (!user) return null;  // Explicit null
    return user;
  },
});
```

---

### Expecting `v.map()` or `v.set()` to Work

**Problem**: JavaScript Map and Set aren't supported.

```typescript
// WRONG: These validators don't exist
defineTable({
  tags: v.set(v.string()),      // NO!
  metadata: v.map(v.string()),  // NO!
})
```

**Solution**: Use arrays or objects.

```typescript
// RIGHT: Use arrays or objects
defineTable({
  tags: v.array(v.string()),  // For Set-like data
  metadata: v.record(v.string(), v.string()),  // For Map-like data
})

// Or for key-value with complex values
defineTable({
  entries: v.array(v.object({
    key: v.string(),
    value: v.any(),
  })),
})
```

---

### Using Deprecated `ctx.storage.getMetadata`

**Problem**: Old API that's been removed.

```typescript
// WRONG: Deprecated
const metadata = await ctx.storage.getMetadata(storageId);
```

**Solution**: Query the `_storage` system table.

```typescript
// RIGHT: Use system table
const metadata = await ctx.db.system.get(storageId);
// Returns: { contentType, sha256, size }
```

---

## React Integration

### Not Handling Loading State

**Problem**: `useQuery` returns `undefined` while loading.

```typescript
// CRASH: data is undefined initially
function UserList() {
  const users = useQuery(api.users.list);
  return users.map(u => <User key={u._id} {...u} />);  // TypeError!
}
```

**Solution**: Check for undefined.

```typescript
// SAFE: Handle loading
function UserList() {
  const users = useQuery(api.users.list);

  if (users === undefined) {
    return <LoadingSpinner />;
  }

  return users.map(u => <User key={u._id} {...u} />);
}
```

---

### Passing Functions Directly to `ctx.runQuery`

**Problem**: Must use function references.

```typescript
// WRONG: Passing the function directly
await ctx.runQuery(getUser, { userId });  // Error!
```

**Solution**: Use the API object.

```typescript
// RIGHT: Use function reference
import { internal } from "./_generated/api";

await ctx.runQuery(internal.users.get, { userId });
```

---

### Expecting Immediate UI Updates After Mutation

**Problem**: Not understanding the reactive flow.

```typescript
// Mutation is called
await sendMessage({ text: "Hello" });

// This might not show the new message immediately if you're
// relying on local state instead of the query
```

**Solution**: Trust the reactive system.

```typescript
// The useQuery subscription will automatically update
const messages = useQuery(api.messages.list, { channelId });

// When mutation completes, Convex pushes update to all subscribers
// No need to manually refetch or update local state
```

---

## Scheduling

### Passing Functions to Cron Jobs

**Problem**: Crons need function references.

```typescript
// WRONG: Passing function directly
crons.interval("cleanup", { hours: 1 }, cleanup, {});
```

**Solution**: Import from `_generated/api`.

```typescript
// RIGHT: Use internal reference
import { internal } from "./_generated/api";

crons.interval("cleanup", { hours: 1 }, internal.maintenance.cleanup, {});
```

---

### Expecting Actions to Auto-Retry

**Problem**: Unlike mutations, actions don't retry automatically.

```typescript
// This action might fail and won't be retried
export const sendEmail = action({
  handler: async (ctx, args) => {
    await sendgrid.send({ /* ... */ });  // Network error = permanent failure
  },
});
```

**Solution**: Implement manual retry logic.

```typescript
export const sendEmail = action({
  handler: async (ctx, args) => {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        await sendgrid.send({ /* ... */ });
        return;  // Success
      } catch (error) {
        lastError = error as Error;
        // Exponential backoff with jitter
        await new Promise(r => setTimeout(r, 1000 * Math.pow(2, attempt) * Math.random()));
      }
    }

    throw lastError;
  },
});
```

---

## Performance

### Queries Reading Frequently-Updating Documents

**Problem**: Every read creates a reactive dependency.

```typescript
// PROBLEMATIC: If globalCounter updates every second,
// this query re-runs every second for ALL subscribers
export const getStats = query({
  handler: async (ctx) => {
    const counter = await ctx.db.get(globalCounterId);
    return { count: counter.value };
  },
});
```

**Solutions**:
1. Reduce update frequency (batch updates)
2. Use separate queries for volatile data
3. Consider if real-time is really needed

```typescript
// BETTER: Separate static and dynamic data
export const getUserProfile = query({/* rarely changes */});
export const getActivityCount = query({/* frequently changes */});

// Client loads profile once, activity separately
const profile = useQuery(api.users.getProfile, { userId });
const activity = useQuery(api.users.getActivityCount, { userId });
```

---

### Loading Related Data in a Loop

**Problem**: N+1 query pattern.

```typescript
// SLOW: N+1 queries
export const getPostsWithAuthors = query({
  handler: async (ctx) => {
    const posts = await ctx.db.query("posts").take(50);

    // 50 separate database reads!
    return Promise.all(posts.map(async (post) => ({
      ...post,
      author: await ctx.db.get(post.authorId),
    })));
  },
});
```

**Solution**: Batch load related documents.

```typescript
// FAST: Batch loading
export const getPostsWithAuthors = query({
  handler: async (ctx) => {
    const posts = await ctx.db.query("posts").take(50);

    // Get unique author IDs
    const authorIds = [...new Set(posts.map(p => p.authorId))];

    // Single batch load
    const authors = await Promise.all(authorIds.map(id => ctx.db.get(id)));
    const authorMap = new Map(authors.filter(Boolean).map(a => [a!._id, a]));

    return posts.map(post => ({
      ...post,
      author: authorMap.get(post.authorId),
    }));
  },
});
```

---

## Authentication

### Not Checking Auth in Every Function

**Problem**: Assuming auth is handled elsewhere.

```typescript
// INSECURE: No auth check
export const deletePost = mutation({
  args: { postId: v.id("posts") },
  handler: async (ctx, args) => {
    await ctx.db.delete(args.postId);  // Anyone can delete any post!
  },
});
```

**Solution**: Always verify authentication and authorization.

```typescript
// SECURE: Check auth and ownership
export const deletePost = mutation({
  args: { postId: v.id("posts") },
  handler: async (ctx, args) => {
    const identity = await ctx.auth.getUserIdentity();
    if (!identity) throw new Error("Not authenticated");

    const post = await ctx.db.get(args.postId);
    if (!post) throw new Error("Post not found");

    // Check ownership
    const user = await ctx.db
      .query("users")
      .withIndex("by_clerk_id", q => q.eq("clerkId", identity.subject))
      .unique();

    if (post.authorId !== user?._id) {
      throw new Error("Not authorized");
    }

    await ctx.db.delete(args.postId);
  },
});
```

---

## Schema Evolution

### Making Fields Required Without Migration

**Problem**: Existing documents don't have the new field.

```typescript
// Step 1: Schema says required
defineTable({
  name: v.string(),
  email: v.string(),  // New required field
})

// CRASH: Old documents fail validation
```

**Solution**: Add as optional, migrate, then require.

```typescript
// Step 1: Add as optional
defineTable({
  name: v.string(),
  email: v.optional(v.string()),
})

// Step 2: Run migration
export const backfillEmails = mutation({
  handler: async (ctx) => {
    const users = await ctx.db
      .query("users")
      .filter(q => q.eq(q.field("email"), undefined))
      .take(100);

    for (const user of users) {
      await ctx.db.patch(user._id, { email: "unknown@example.com" });
    }

    return users.length;  // Call repeatedly until returns 0
  },
});

// Step 3: After migration complete, make required
defineTable({
  name: v.string(),
  email: v.string(),
})
```

---

## Common Error Messages

### "Mutation cannot have side effects"

You're trying to do something non-deterministic in a mutation. Move external calls to an action.

### "Query exceeded document limit"

You're reading too many documents. Add indexes, use pagination, or narrow your query.

### "Function reference expected"

You're passing a function directly instead of using the `api` or `internal` object.

### "undefined is not a valid Convex value"

Replace `undefined` with `null` or use `v.optional()`.

### "Index not found"

You referenced an index that doesn't exist. Check your schema.ts file.
