# Function Patterns

Advanced patterns for Convex queries, mutations, actions, and helper functions.

## Helper Functions

### When to Use Plain Functions vs ctx.runQuery

```typescript
// PREFER: Plain TypeScript helper functions
async function getUserByEmail(ctx: QueryCtx, email: string) {
  return await ctx.db
    .query("users")
    .withIndex("by_email", (q) => q.eq("email", email))
    .unique();
}

export const getProfile = query({
  args: { email: v.string() },
  handler: async (ctx, args) => {
    const user = await getUserByEmail(ctx, args.email);  // Direct call
    return user;
  },
});
```

```typescript
// AVOID: ctx.runQuery for internal reuse
export const getProfile = query({
  handler: async (ctx, args) => {
    // Unnecessary overhead
    const user = await ctx.runQuery(api.users.getByEmail, { email: args.email });
  },
});
```

**Use `ctx.runQuery`/`ctx.runMutation` only when**:
- Calling component functions
- Need partial rollback on error (separate transaction)

### Context Type Helpers

```typescript
// convex/helpers.ts
import { QueryCtx, MutationCtx, ActionCtx } from "./_generated/server";
import { Doc, Id } from "./_generated/dataModel";

// Read-only helper (works in query or mutation)
export async function getUser(
  ctx: QueryCtx,
  userId: Id<"users">
): Promise<Doc<"users"> | null> {
  return await ctx.db.get(userId);
}

// Write helper (mutation only)
export async function updateUserName(
  ctx: MutationCtx,
  userId: Id<"users">,
  name: string
): Promise<void> {
  await ctx.db.patch(userId, { name });
}

// Auth helper
export async function requireAuth(ctx: QueryCtx | MutationCtx) {
  const identity = await ctx.auth.getUserIdentity();
  if (!identity) {
    throw new Error("Not authenticated");
  }
  return identity;
}

export async function requireUser(ctx: QueryCtx | MutationCtx) {
  const identity = await requireAuth(ctx);

  const user = await ctx.db
    .query("users")
    .withIndex("by_clerk_id", (q) => q.eq("clerkId", identity.subject))
    .unique();

  if (!user) {
    throw new Error("User not found");
  }

  return user;
}
```

## Custom Function Builders

Using `convex-helpers` for middleware patterns:

```typescript
// convex/functions.ts
import { customQuery, customMutation } from "convex-helpers/server/customFunctions";
import { query, mutation } from "./_generated/server";

// Authenticated query builder
export const authenticatedQuery = customQuery(query, {
  args: {},
  input: async (ctx, args) => {
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

    return { ctx: { ...ctx, user }, args };
  },
});

// Usage
export const getMyProfile = authenticatedQuery({
  args: {},
  handler: async (ctx) => {
    // ctx.user is available and typed!
    return ctx.user;
  },
});
```

### Admin-Only Functions

```typescript
export const adminMutation = customMutation(mutation, {
  args: {},
  input: async (ctx, args) => {
    const identity = await ctx.auth.getUserIdentity();
    if (!identity) throw new Error("Not authenticated");

    const user = await ctx.db
      .query("users")
      .withIndex("by_clerk_id", (q) => q.eq("clerkId", identity.subject))
      .unique();

    if (!user || user.role !== "admin") {
      throw new Error("Admin access required");
    }

    return { ctx: { ...ctx, user }, args };
  },
});

export const deleteUser = adminMutation({
  args: { userId: v.id("users") },
  handler: async (ctx, args) => {
    await ctx.db.delete(args.userId);
  },
});
```

## Query Patterns

### Efficient Loading with Relations

```typescript
export const getChannelWithMessages = query({
  args: { channelId: v.id("channels") },
  handler: async (ctx, args) => {
    const channel = await ctx.db.get(args.channelId);
    if (!channel) return null;

    const messages = await ctx.db
      .query("messages")
      .withIndex("by_channel", (q) => q.eq("channelId", args.channelId))
      .order("desc")
      .take(50);

    // Load authors in parallel
    const authorIds = [...new Set(messages.map((m) => m.authorId))];
    const authors = await Promise.all(authorIds.map((id) => ctx.db.get(id)));
    const authorMap = new Map(authors.filter(Boolean).map((a) => [a!._id, a]));

    return {
      channel,
      messages: messages.map((m) => ({
        ...m,
        author: authorMap.get(m.authorId),
      })),
    };
  },
});
```

### Conditional Loading

```typescript
export const getUser = query({
  args: {
    userId: v.id("users"),
    includeStats: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const user = await ctx.db.get(args.userId);
    if (!user) return null;

    if (!args.includeStats) {
      return { user, stats: null };
    }

    // Only load stats if requested
    const postCount = await ctx.db
      .query("posts")
      .withIndex("by_author", (q) => q.eq("authorId", args.userId))
      .collect()
      .then((posts) => posts.length);

    return {
      user,
      stats: { postCount },
    };
  },
});
```

### Search Queries

```typescript
// Define search index in schema
defineTable({
  title: v.string(),
  content: v.string(),
  authorId: v.id("users"),
}).searchIndex("search_content", {
  searchField: "content",
  filterFields: ["authorId"],
});

// Search query
export const searchPosts = query({
  args: {
    query: v.string(),
    authorId: v.optional(v.id("users")),
  },
  handler: async (ctx, args) => {
    let searchQuery = ctx.db
      .query("posts")
      .withSearchIndex("search_content", (q) => q.search("content", args.query));

    if (args.authorId) {
      searchQuery = searchQuery.filter((q) =>
        q.eq(q.field("authorId"), args.authorId)
      );
    }

    return await searchQuery.take(20);
  },
});
```

## Mutation Patterns

### Idempotent Mutations

```typescript
export const likePost = mutation({
  args: { postId: v.id("posts") },
  handler: async (ctx, args) => {
    const user = await requireUser(ctx);

    // Check if already liked (idempotent)
    const existingLike = await ctx.db
      .query("likes")
      .withIndex("by_user_and_post", (q) =>
        q.eq("userId", user._id).eq("postId", args.postId)
      )
      .unique();

    if (existingLike) {
      return existingLike._id;  // Already liked, return existing
    }

    return await ctx.db.insert("likes", {
      userId: user._id,
      postId: args.postId,
    });
  },
});
```

### Soft Delete

```typescript
export const deletePost = mutation({
  args: { postId: v.id("posts") },
  handler: async (ctx, args) => {
    const user = await requireUser(ctx);
    const post = await ctx.db.get(args.postId);

    if (!post) throw new Error("Post not found");
    if (post.authorId !== user._id) throw new Error("Not authorized");

    // Soft delete
    await ctx.db.patch(args.postId, {
      deletedAt: Date.now(),
      deletedBy: user._id,
    });
  },
});

// Query excludes soft-deleted
export const listPosts = query({
  handler: async (ctx) => {
    return await ctx.db
      .query("posts")
      .filter((q) => q.eq(q.field("deletedAt"), undefined))
      .order("desc")
      .take(50);
  },
});
```

### Batch Operations

```typescript
export const deleteMessages = mutation({
  args: { messageIds: v.array(v.id("messages")) },
  handler: async (ctx, args) => {
    const user = await requireUser(ctx);

    // Validate all messages belong to user
    const messages = await Promise.all(
      args.messageIds.map((id) => ctx.db.get(id))
    );

    for (const msg of messages) {
      if (!msg) throw new Error("Message not found");
      if (msg.authorId !== user._id) throw new Error("Not authorized");
    }

    // Delete all
    await Promise.all(args.messageIds.map((id) => ctx.db.delete(id)));

    return { deleted: args.messageIds.length };
  },
});
```

## Action Patterns

### Reliable Action with Retry

```typescript
// convex/lib/retry.ts
export async function withRetry<T>(
  fn: () => Promise<T>,
  options: { maxAttempts?: number; baseDelay?: number } = {}
): Promise<T> {
  const { maxAttempts = 3, baseDelay = 1000 } = options;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      if (attempt === maxAttempts) throw error;

      // Exponential backoff with jitter
      const delay = baseDelay * Math.pow(2, attempt - 1) * (0.5 + Math.random());
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }

  throw new Error("Unreachable");
}

// Usage in action
export const sendEmail = action({
  args: { to: v.string(), subject: v.string(), body: v.string() },
  handler: async (ctx, args) => {
    await withRetry(async () => {
      await sendgrid.send({
        to: args.to,
        subject: args.subject,
        html: args.body,
      });
    });
  },
});
```

### Action with Progress Tracking

```typescript
export const processLargeFile = action({
  args: { jobId: v.id("jobs"), fileUrl: v.string() },
  handler: async (ctx, args) => {
    // Update status
    await ctx.runMutation(internal.jobs.updateStatus, {
      jobId: args.jobId,
      status: "processing",
    });

    try {
      const data = await fetch(args.fileUrl).then((r) => r.text());
      const lines = data.split("\n");

      for (let i = 0; i < lines.length; i++) {
        // Process line...

        // Update progress every 100 lines
        if (i % 100 === 0) {
          await ctx.runMutation(internal.jobs.updateProgress, {
            jobId: args.jobId,
            progress: Math.round((i / lines.length) * 100),
          });
        }
      }

      await ctx.runMutation(internal.jobs.updateStatus, {
        jobId: args.jobId,
        status: "completed",
      });
    } catch (error) {
      await ctx.runMutation(internal.jobs.updateStatus, {
        jobId: args.jobId,
        status: "failed",
        error: String(error),
      });
      throw error;
    }
  },
});
```

### Parallel External Calls

```typescript
export const enrichUserData = action({
  args: { userId: v.id("users") },
  handler: async (ctx, args) => {
    const user = await ctx.runQuery(internal.users.get, { userId: args.userId });
    if (!user) throw new Error("User not found");

    // Parallel external API calls
    const [githubData, linkedinData, twitterData] = await Promise.allSettled([
      fetchGithubProfile(user.githubUsername),
      fetchLinkedinProfile(user.linkedinUrl),
      fetchTwitterProfile(user.twitterHandle),
    ]);

    // Update with whatever succeeded
    await ctx.runMutation(internal.users.updateEnrichment, {
      userId: args.userId,
      github: githubData.status === "fulfilled" ? githubData.value : null,
      linkedin: linkedinData.status === "fulfilled" ? linkedinData.value : null,
      twitter: twitterData.status === "fulfilled" ? twitterData.value : null,
    });
  },
});
```

## HTTP Endpoints

```typescript
// convex/http.ts
import { httpRouter } from "convex/server";
import { httpAction } from "./_generated/server";

const http = httpRouter();

// Webhook handler
http.route({
  path: "/webhooks/stripe",
  method: "POST",
  handler: httpAction(async (ctx, request) => {
    const signature = request.headers.get("stripe-signature");
    const body = await request.text();

    // Verify webhook signature
    const event = stripe.webhooks.constructEvent(body, signature!, webhookSecret);

    // Process event
    await ctx.runMutation(internal.payments.processWebhook, { event });

    return new Response(null, { status: 200 });
  }),
});

// API endpoint
http.route({
  path: "/api/public/stats",
  method: "GET",
  handler: httpAction(async (ctx, request) => {
    const stats = await ctx.runQuery(api.stats.getPublic);

    return new Response(JSON.stringify(stats), {
      headers: { "Content-Type": "application/json" },
    });
  }),
});

export default http;
```

## Argument Validation Patterns

### Reusable Argument Objects

```typescript
// convex/validators.ts
export const paginationArgs = {
  cursor: v.optional(v.string()),
  limit: v.optional(v.number()),
};

export const withPagination = <T extends Record<string, any>>(args: T) => ({
  ...args,
  ...paginationArgs,
});

// Usage
export const listMessages = query({
  args: withPagination({
    channelId: v.id("channels"),
  }),
  handler: async (ctx, args) => {
    // args.channelId, args.cursor, args.limit all typed
  },
});
```

### Table-Derived Validators

```typescript
import { v } from "convex/values";

// Define fields once
export const userFields = {
  name: v.string(),
  email: v.string(),
  role: v.union(v.literal("admin"), v.literal("member")),
};

// Use in schema
defineTable(userFields).index("by_email", ["email"]);

// Use in function args
export const createUser = mutation({
  args: userFields,
  handler: async (ctx, args) => {
    return await ctx.db.insert("users", args);
  },
});

// Partial for updates
export const updateUser = mutation({
  args: {
    userId: v.id("users"),
    updates: v.object(partial(userFields)),
  },
  handler: async (ctx, args) => {
    await ctx.db.patch(args.userId, args.updates);
  },
});
```

## Error Handling

### Typed Errors

```typescript
export class ConvexError extends Error {
  constructor(
    public code: "NOT_FOUND" | "UNAUTHORIZED" | "FORBIDDEN" | "VALIDATION",
    message: string
  ) {
    super(message);
    this.name = "ConvexError";
  }
}

export const getPost = query({
  args: { postId: v.id("posts") },
  handler: async (ctx, args) => {
    const post = await ctx.db.get(args.postId);

    if (!post) {
      throw new ConvexError("NOT_FOUND", "Post not found");
    }

    if (post.isPrivate) {
      const user = await ctx.auth.getUserIdentity();
      if (!user || post.authorId !== user.subject) {
        throw new ConvexError("FORBIDDEN", "Cannot access private post");
      }
    }

    return post;
  },
});
```

### Client-Side Error Handling

```typescript
// React component
function CreatePost() {
  const createPost = useMutation(api.posts.create);

  const handleSubmit = async () => {
    try {
      await createPost({ title, content });
    } catch (error) {
      if (error instanceof ConvexError) {
        if (error.data?.code === "VALIDATION") {
          setError(error.message);
        } else if (error.data?.code === "UNAUTHORIZED") {
          router.push("/login");
        }
      } else {
        setError("Something went wrong");
      }
    }
  };
}
```

## Transaction Patterns

### Optimistic Locking

```typescript
export const updatePost = mutation({
  args: {
    postId: v.id("posts"),
    expectedVersion: v.number(),
    updates: v.object({ title: v.string(), content: v.string() }),
  },
  handler: async (ctx, args) => {
    const post = await ctx.db.get(args.postId);

    if (!post) throw new Error("Post not found");

    if (post.version !== args.expectedVersion) {
      throw new Error("Post was modified by another user");
    }

    await ctx.db.patch(args.postId, {
      ...args.updates,
      version: post.version + 1,
    });
  },
});
```

### Atomic Counter

```typescript
export const incrementViews = mutation({
  args: { postId: v.id("posts") },
  handler: async (ctx, args) => {
    const post = await ctx.db.get(args.postId);
    if (!post) throw new Error("Post not found");

    // Atomic increment in transaction
    await ctx.db.patch(args.postId, {
      viewCount: (post.viewCount ?? 0) + 1,
    });
  },
});
```
