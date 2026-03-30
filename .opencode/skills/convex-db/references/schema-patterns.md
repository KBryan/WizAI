# Schema Design Patterns

Advanced schema design patterns for Convex applications.

## Relationship Patterns

### One-to-Many Relationships

Store the "one" side's ID on the "many" side:

```typescript
// schema.ts
export default defineSchema({
  channels: defineTable({
    name: v.string(),
    description: v.optional(v.string()),
  }),

  messages: defineTable({
    channelId: v.id("channels"),  // Reference to parent
    text: v.string(),
    authorId: v.id("users"),
  }).index("by_channel", ["channelId"]),
});

// Query messages in a channel
const messages = await ctx.db
  .query("messages")
  .withIndex("by_channel", (q) => q.eq("channelId", channelId))
  .collect();
```

### Many-to-Many Relationships

Use a junction table:

```typescript
export default defineSchema({
  users: defineTable({
    name: v.string(),
  }),

  teams: defineTable({
    name: v.string(),
  }),

  // Junction table
  teamMembers: defineTable({
    userId: v.id("users"),
    teamId: v.id("teams"),
    role: v.union(v.literal("admin"), v.literal("member")),
    joinedAt: v.number(),
  })
    .index("by_user", ["userId"])
    .index("by_team", ["teamId"])
    .index("by_user_and_team", ["userId", "teamId"]),
});

// Get all teams for a user
const memberships = await ctx.db
  .query("teamMembers")
  .withIndex("by_user", (q) => q.eq("userId", userId))
  .collect();

const teams = await Promise.all(
  memberships.map((m) => ctx.db.get(m.teamId))
);

// Check if user is in team
const membership = await ctx.db
  .query("teamMembers")
  .withIndex("by_user_and_team", (q) =>
    q.eq("userId", userId).eq("teamId", teamId)
  )
  .unique();
```

### One-to-One Relationships

Two patterns depending on optionality:

```typescript
// Pattern 1: Optional extension (separate table)
export default defineSchema({
  users: defineTable({
    name: v.string(),
    email: v.string(),
  }),

  userPreferences: defineTable({
    userId: v.id("users"),  // Unique per user
    theme: v.string(),
    notifications: v.boolean(),
  }).index("by_user", ["userId"]),
});

// Pattern 2: Embedded (single table)
export default defineSchema({
  users: defineTable({
    name: v.string(),
    email: v.string(),
    preferences: v.optional(v.object({
      theme: v.string(),
      notifications: v.boolean(),
    })),
  }),
});
```

## Index Design

### Compound Indexes for Filtering + Ordering

```typescript
defineTable({
  channelId: v.id("channels"),
  authorId: v.id("users"),
  createdAt: v.number(),
  isPinned: v.boolean(),
})
  // Filter by channel, order by time
  .index("by_channel_time", ["channelId", "createdAt"])

  // Filter by channel AND author
  .index("by_channel_author", ["channelId", "authorId"])

  // Filter by pinned status in channel
  .index("by_channel_pinned", ["channelId", "isPinned", "createdAt"])
```

### Index Field Ordering

**Rule**: Equality fields first, range/order fields last.

```typescript
// Query: Get messages in channel X, ordered by time
// Index should be: ["channelId", "createdAt"]
//                   ^^^equality    ^^^range/order

await ctx.db
  .query("messages")
  .withIndex("by_channel_time", (q) =>
    q.eq("channelId", channelId)  // Equality
  )
  .order("desc")  // Orders by createdAt (last index field)
  .take(50);
```

### Avoiding Redundant Indexes

```typescript
// REDUNDANT: by_channel is subset of by_channel_time
.index("by_channel", ["channelId"])
.index("by_channel_time", ["channelId", "createdAt"])

// BETTER: Just keep the compound index
.index("by_channel_time", ["channelId", "createdAt"])

// You can still query by just channelId:
.withIndex("by_channel_time", (q) => q.eq("channelId", id))
```

## Validator Patterns

### Reusable Validators

```typescript
// convex/validators.ts
import { v } from "convex/values";

export const userRole = v.union(
  v.literal("admin"),
  v.literal("moderator"),
  v.literal("member")
);

export const messageContent = v.object({
  text: v.string(),
  attachments: v.optional(v.array(v.id("_storage"))),
  mentions: v.optional(v.array(v.id("users"))),
});

export const paginationArgs = {
  cursor: v.optional(v.string()),
  limit: v.optional(v.number()),
};
```

```typescript
// convex/schema.ts
import { userRole, messageContent } from "./validators";

export default defineSchema({
  users: defineTable({
    name: v.string(),
    role: userRole,
  }),

  messages: defineTable({
    channelId: v.id("channels"),
    content: messageContent,
  }),
});
```

### Discriminated Unions

```typescript
const notificationContent = v.union(
  v.object({
    type: v.literal("message"),
    messageId: v.id("messages"),
    senderId: v.id("users"),
  }),
  v.object({
    type: v.literal("mention"),
    messageId: v.id("messages"),
    mentionedBy: v.id("users"),
  }),
  v.object({
    type: v.literal("system"),
    title: v.string(),
    body: v.string(),
  })
);

defineTable({
  userId: v.id("users"),
  content: notificationContent,
  read: v.boolean(),
  createdAt: v.number(),
});
```

### Optional vs Nullable

```typescript
// Optional: Field may be omitted entirely
v.optional(v.string())
// TypeScript: string | undefined
// Can omit when inserting

// Nullable: Field must exist but can be null
v.union(v.string(), v.null())
// TypeScript: string | null
// Must provide value (string or null)

// Optional AND nullable
v.optional(v.union(v.string(), v.null()))
// TypeScript: string | null | undefined
```

## System Fields

Every document has system-managed fields:

```typescript
{
  _id: Id<"tableName">,      // Unique document ID
  _creationTime: number,     // Millisecond timestamp
  // ... your fields
}
```

**Notes**:
- Default ordering is by `_creationTime` ascending
- Use `_id` for lookups when possible (most efficient)
- `_creationTime` is automatically indexed

## Denormalization Patterns

### Cached Counts

Instead of counting on every query:

```typescript
// Store count on parent
defineTable({
  name: v.string(),
  memberCount: v.number(),  // Denormalized count
})

// Update count in mutation
export const addMember = mutation({
  handler: async (ctx, args) => {
    await ctx.db.insert("members", { teamId: args.teamId, userId: args.userId });

    // Update count
    const team = await ctx.db.get(args.teamId);
    await ctx.db.patch(args.teamId, {
      memberCount: (team?.memberCount ?? 0) + 1,
    });
  },
});
```

### Embedded Recent Items

```typescript
// Store last N items directly on parent
defineTable({
  name: v.string(),
  recentMessages: v.array(v.object({
    id: v.id("messages"),
    preview: v.string(),
    authorName: v.string(),
    createdAt: v.number(),
  })),
})

// Update on new message
export const sendMessage = mutation({
  handler: async (ctx, args) => {
    const messageId = await ctx.db.insert("messages", { /* ... */ });

    const channel = await ctx.db.get(args.channelId);
    const recentMessages = [
      { id: messageId, preview: args.text.slice(0, 100), authorName, createdAt },
      ...(channel?.recentMessages ?? []).slice(0, 4),  // Keep last 5
    ];

    await ctx.db.patch(args.channelId, { recentMessages });
  },
});
```

## Migration Patterns

### Adding Required Fields

```typescript
// 1. Add as optional first
defineTable({
  name: v.string(),
  newField: v.optional(v.string()),  // Optional initially
})

// 2. Backfill existing documents
export const backfillNewField = mutation({
  handler: async (ctx) => {
    const docs = await ctx.db.query("myTable").take(100);
    for (const doc of docs) {
      if (doc.newField === undefined) {
        await ctx.db.patch(doc._id, { newField: "default value" });
      }
    }
    return docs.length;  // Return count for pagination
  },
});

// 3. After backfill complete, make required
defineTable({
  name: v.string(),
  newField: v.string(),  // Now required
})
```

### Renaming Fields

```typescript
// 1. Add new field, keep old
defineTable({
  oldName: v.optional(v.string()),
  newName: v.optional(v.string()),
})

// 2. Migrate data
export const migrateField = mutation({
  handler: async (ctx) => {
    const docs = await ctx.db
      .query("myTable")
      .filter((q) => q.neq(q.field("oldName"), undefined))
      .take(100);

    for (const doc of docs) {
      await ctx.db.patch(doc._id, {
        newName: doc.oldName,
        oldName: undefined,
      });
    }
    return docs.length;
  },
});

// 3. Remove old field from schema
defineTable({
  newName: v.string(),
})
```

## Using convex-helpers

The `convex-helpers` library provides utilities:

```typescript
import { pick, omit } from "convex-helpers";
import { partial } from "convex-helpers/validators";

// Pick specific fields from validator
const nameAndEmail = pick(userFields, ["name", "email"]);

// Omit fields
const withoutPassword = omit(userFields, ["passwordHash"]);

// Make all fields optional (for patches)
const userPatch = partial(userFields);
```

## Table Naming Conventions

- Use **plural nouns**: `users`, `messages`, `channels`
- Use **camelCase**: `teamMembers`, `userPreferences`
- Junction tables: combine both table names: `userTeams`, `postTags`
- System tables start with underscore: `_storage`, `_scheduled_functions`
