# Integration Patterns

React, Next.js, and authentication patterns for Convex applications.

## React Setup

### Basic Provider

```tsx
// main.tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ConvexProvider, ConvexReactClient } from "convex/react";
import App from "./App";

const convex = new ConvexReactClient(import.meta.env.VITE_CONVEX_URL);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ConvexProvider client={convex}>
      <App />
    </ConvexProvider>
  </StrictMode>
);
```

### Environment Variables

```bash
# .env.local (Vite)
VITE_CONVEX_URL=https://your-project.convex.cloud

# .env.local (Next.js)
NEXT_PUBLIC_CONVEX_URL=https://your-project.convex.cloud
```

## React Hooks

### useQuery - Real-time Data

```tsx
import { useQuery } from "convex/react";
import { api } from "../convex/_generated/api";

function MessageList({ channelId }: { channelId: Id<"channels"> }) {
  // Returns undefined while loading, then data
  const messages = useQuery(api.messages.list, { channelId });

  if (messages === undefined) {
    return <LoadingSpinner />;
  }

  if (messages.length === 0) {
    return <EmptyState message="No messages yet" />;
  }

  return (
    <ul>
      {messages.map((msg) => (
        <li key={msg._id}>{msg.text}</li>
      ))}
    </ul>
  );
}
```

### Conditional Queries

```tsx
// Skip query when value is undefined
function UserProfile({ userId }: { userId?: Id<"users"> }) {
  // Query only runs when userId is defined
  const user = useQuery(
    api.users.get,
    userId ? { userId } : "skip"
  );

  if (userId === undefined) {
    return <p>Select a user</p>;
  }

  if (user === undefined) {
    return <LoadingSpinner />;
  }

  return <Profile user={user} />;
}
```

### useMutation - Data Changes

```tsx
import { useMutation } from "convex/react";
import { api } from "../convex/_generated/api";

function CreatePost() {
  const createPost = useMutation(api.posts.create);
  const [title, setTitle] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const postId = await createPost({ title });
      router.push(`/posts/${postId}`);
    } catch (error) {
      toast.error("Failed to create post");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        disabled={isLoading}
      />
      <button type="submit" disabled={isLoading}>
        {isLoading ? "Creating..." : "Create"}
      </button>
    </form>
  );
}
```

### useAction - External Operations

```tsx
import { useAction } from "convex/react";
import { api } from "../convex/_generated/api";

function AIAssistant() {
  const generateResponse = useAction(api.ai.generate);
  const [response, setResponse] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleGenerate = async () => {
    setIsLoading(true);
    try {
      const result = await generateResponse({ prompt: "Hello!" });
      setResponse(result);
    } catch (error) {
      toast.error("AI generation failed");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div>
      <button onClick={handleGenerate} disabled={isLoading}>
        Generate
      </button>
      {response && <p>{response}</p>}
    </div>
  );
}
```

### usePaginatedQuery - Infinite Scroll

```tsx
import { usePaginatedQuery } from "convex/react";
import { api } from "../convex/_generated/api";

function InfinitePostList() {
  const { results, status, loadMore } = usePaginatedQuery(
    api.posts.list,
    {},
    { initialNumItems: 20 }
  );

  return (
    <div>
      {results.map((post) => (
        <PostCard key={post._id} post={post} />
      ))}

      {status === "LoadingMore" && <LoadingSpinner />}

      {status === "CanLoadMore" && (
        <button onClick={() => loadMore(20)}>Load more</button>
      )}

      {status === "Exhausted" && <p>No more posts</p>}
    </div>
  );
}
```

### Optimistic Updates

```tsx
import { useMutation, useQueryClient } from "convex/react";

function LikeButton({ postId }: { postId: Id<"posts"> }) {
  const queryClient = useQueryClient();
  const toggleLike = useMutation(api.posts.toggleLike);

  const handleLike = async () => {
    // Optimistic update
    queryClient.setQueryData(
      api.posts.get,
      { postId },
      (old) => old ? { ...old, liked: !old.liked } : old
    );

    try {
      await toggleLike({ postId });
    } catch {
      // Revert on error - Convex will also push correct state
      queryClient.invalidateQueries(api.posts.get, { postId });
    }
  };

  return <button onClick={handleLike}>Like</button>;
}
```

## Next.js App Router

### Provider Setup

```tsx
// app/providers.tsx
"use client";

import { ConvexProvider, ConvexReactClient } from "convex/react";
import { ReactNode } from "react";

const convex = new ConvexReactClient(process.env.NEXT_PUBLIC_CONVEX_URL!);

export function Providers({ children }: { children: ReactNode }) {
  return <ConvexProvider client={convex}>{children}</ConvexProvider>;
}
```

```tsx
// app/layout.tsx
import { Providers } from "./providers";

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html>
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
```

### Server Components with Preloading

```tsx
// app/posts/[id]/page.tsx
import { preloadQuery } from "convex/nextjs";
import { api } from "@/convex/_generated/api";
import { PostContent } from "./PostContent";

export default async function PostPage({ params }: { params: { id: string } }) {
  // Preload on server
  const preloadedPost = await preloadQuery(api.posts.get, {
    postId: params.id as Id<"posts">,
  });

  return <PostContent preloadedPost={preloadedPost} />;
}
```

```tsx
// app/posts/[id]/PostContent.tsx
"use client";

import { usePreloadedQuery } from "convex/react";
import { Preloaded } from "convex/react";
import { api } from "@/convex/_generated/api";

export function PostContent({
  preloadedPost,
}: {
  preloadedPost: Preloaded<typeof api.posts.get>;
}) {
  // Hydrates from preloaded data, then subscribes to real-time updates
  const post = usePreloadedQuery(preloadedPost);

  if (!post) return <NotFound />;

  return <article>{post.title}</article>;
}
```

### Server-Only Fetching

```tsx
// For data that doesn't need real-time updates
import { fetchQuery } from "convex/nextjs";
import { api } from "@/convex/_generated/api";

export default async function StatsPage() {
  // One-time fetch, no subscription
  const stats = await fetchQuery(api.stats.getPublic);

  return (
    <div>
      <p>Total users: {stats.userCount}</p>
    </div>
  );
}
```

### Server Actions with Mutations

```tsx
// app/actions.ts
"use server";

import { fetchMutation } from "convex/nextjs";
import { api } from "@/convex/_generated/api";

export async function createPost(formData: FormData) {
  const title = formData.get("title") as string;

  const postId = await fetchMutation(api.posts.create, { title });

  redirect(`/posts/${postId}`);
}
```

```tsx
// app/posts/new/page.tsx
import { createPost } from "../actions";

export default function NewPostPage() {
  return (
    <form action={createPost}>
      <input name="title" required />
      <button type="submit">Create</button>
    </form>
  );
}
```

## Authentication with Clerk

### Setup

```bash
npm install @clerk/nextjs
```

```tsx
// app/providers.tsx
"use client";

import { ClerkProvider, useAuth } from "@clerk/nextjs";
import { ConvexProviderWithClerk } from "convex/react-clerk";
import { ConvexReactClient } from "convex/react";

const convex = new ConvexReactClient(process.env.NEXT_PUBLIC_CONVEX_URL!);

export function Providers({ children }: { children: ReactNode }) {
  return (
    <ClerkProvider>
      <ConvexProviderWithClerk client={convex} useAuth={useAuth}>
        {children}
      </ConvexProviderWithClerk>
    </ClerkProvider>
  );
}
```

### Convex Auth Configuration

```typescript
// convex/auth.config.ts
export default {
  providers: [
    {
      domain: "https://your-clerk-domain.clerk.accounts.dev",
      applicationID: "convex",
    },
  ],
};
```

### Accessing User Identity

```typescript
// In Convex functions
export const getCurrentUser = query({
  handler: async (ctx) => {
    const identity = await ctx.auth.getUserIdentity();

    if (!identity) {
      return null;
    }

    // identity.subject = Clerk user ID
    // identity.name, identity.email, etc.
    return await ctx.db
      .query("users")
      .withIndex("by_clerk_id", (q) => q.eq("clerkId", identity.subject))
      .unique();
  },
});
```

### Webhook for User Sync

```typescript
// convex/http.ts
import { httpRouter } from "convex/server";
import { httpAction } from "./_generated/server";
import { Webhook } from "svix";

const http = httpRouter();

http.route({
  path: "/clerk-webhook",
  method: "POST",
  handler: httpAction(async (ctx, request) => {
    const webhookSecret = process.env.CLERK_WEBHOOK_SECRET!;
    const svix_id = request.headers.get("svix-id")!;
    const svix_timestamp = request.headers.get("svix-timestamp")!;
    const svix_signature = request.headers.get("svix-signature")!;

    const body = await request.text();

    const wh = new Webhook(webhookSecret);
    const evt = wh.verify(body, {
      "svix-id": svix_id,
      "svix-timestamp": svix_timestamp,
      "svix-signature": svix_signature,
    }) as WebhookEvent;

    if (evt.type === "user.created") {
      await ctx.runMutation(internal.users.create, {
        clerkId: evt.data.id,
        email: evt.data.email_addresses[0].email_address,
        name: `${evt.data.first_name} ${evt.data.last_name}`,
      });
    }

    if (evt.type === "user.updated") {
      await ctx.runMutation(internal.users.update, {
        clerkId: evt.data.id,
        email: evt.data.email_addresses[0].email_address,
        name: `${evt.data.first_name} ${evt.data.last_name}`,
      });
    }

    if (evt.type === "user.deleted") {
      await ctx.runMutation(internal.users.delete, {
        clerkId: evt.data.id,
      });
    }

    return new Response(null, { status: 200 });
  }),
});

export default http;
```

### Protected Components

```tsx
"use client";

import { useUser } from "@clerk/nextjs";
import { useQuery } from "convex/react";
import { api } from "@/convex/_generated/api";

function Dashboard() {
  const { isLoaded, isSignedIn } = useUser();
  const userData = useQuery(api.users.getCurrentUser);

  if (!isLoaded) {
    return <LoadingSpinner />;
  }

  if (!isSignedIn) {
    return <SignInButton />;
  }

  if (userData === undefined) {
    return <LoadingSpinner />;
  }

  return <DashboardContent user={userData} />;
}
```

## File Upload Pattern

```tsx
"use client";

import { useMutation } from "convex/react";
import { api } from "@/convex/_generated/api";

function ImageUpload({ onUpload }: { onUpload: (storageId: Id<"_storage">) => void }) {
  const generateUploadUrl = useMutation(api.files.generateUploadUrl);
  const [isUploading, setIsUploading] = useState(false);

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setIsUploading(true);

    try {
      // Get upload URL from Convex
      const uploadUrl = await generateUploadUrl();

      // Upload file directly to Convex storage
      const result = await fetch(uploadUrl, {
        method: "POST",
        headers: { "Content-Type": file.type },
        body: file,
      });

      const { storageId } = await result.json();
      onUpload(storageId);
    } catch (error) {
      toast.error("Upload failed");
    } finally {
      setIsUploading(false);
    }
  };

  return (
    <input
      type="file"
      accept="image/*"
      onChange={handleFileChange}
      disabled={isUploading}
    />
  );
}
```

## Loading States Pattern

```tsx
// Centralized loading component
function QueryLoader<T>({
  query,
  args,
  children,
  fallback = <LoadingSpinner />,
}: {
  query: FunctionReference<"query">;
  args: Record<string, unknown>;
  children: (data: T) => ReactNode;
  fallback?: ReactNode;
}) {
  const data = useQuery(query, args);

  if (data === undefined) {
    return fallback;
  }

  return children(data as T);
}

// Usage
<QueryLoader query={api.posts.get} args={{ postId }}>
  {(post) => <PostCard post={post} />}
</QueryLoader>
```

## Error Boundary Pattern

```tsx
"use client";

import { ErrorBoundary } from "react-error-boundary";

function ConvexErrorFallback({ error, resetErrorBoundary }) {
  return (
    <div className="error-container">
      <h2>Something went wrong</h2>
      <p>{error.message}</p>
      <button onClick={resetErrorBoundary}>Try again</button>
    </div>
  );
}

function App() {
  return (
    <ErrorBoundary
      FallbackComponent={ConvexErrorFallback}
      onReset={() => {
        // Reset app state
      }}
    >
      <ConvexProvider client={convex}>
        <YourApp />
      </ConvexProvider>
    </ErrorBoundary>
  );
}
```

## Caching with convex-helpers

```tsx
// Persistent query subscriptions across navigation
import { useQuery } from "convex-helpers/react/cache/hooks";

function Component() {
  // This subscription persists when navigating away and back
  const data = useQuery(api.data.get, { id });
}
```

## Real-time Presence

```typescript
// Using convex-helpers presence
import { usePresence, usePresenceContext } from "convex-helpers/react/presence";

function CollaborativeEditor({ docId }: { docId: Id<"docs"> }) {
  const presence = usePresence("editors", docId);

  // Show who else is editing
  const otherUsers = presence.others.map((p) => p.user);

  // Update your cursor position
  useEffect(() => {
    presence.update({ cursor: cursorPosition });
  }, [cursorPosition]);
}
```
