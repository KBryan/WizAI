# TypeScript Developer Skill

## Overview

This skill enables the agent to write professional TypeScript code for full-stack web applications.

## When to Use

- Building React/Vue frontend applications
- Writing Node.js backend APIs
- Creating type-safe code with proper interfaces
- Generating TypeScript from specifications

## TypeScript Best Practices

### Type Safety
- Always define interfaces for data structures
- Use explicit return types on functions
- Avoid `any` type - use `unknown` with type guards instead
- Leverage TypeScript's strict mode

### Code Structure
```typescript
// Types first
interface User {
  id: string;
  name: string;
  email: string;
}

// Pure functions with types
function createUser(data: UserInput): User {
  // implementation
}

// Classes for complex state
class UserService {
  private users: Map<string, User> = new Map();
  
  async create(userData: UserInput): Promise<User> {
    // implementation
  }
}
```

### Error Handling
```typescript
type Result<T, E = Error> = 
  | { success: true; data: T }
  | { success: false; error: E };

async function fetchData(): Promise<Result<Data>> {
  try {
    const response = await fetch('/api/data');
    if (!response.ok) throw new Error('HTTP error');
    const data = await response.json();
    return { success: true, data };
  } catch (error) {
    return { success: false, error: error as Error };
  }
}
```

## React Patterns

### Functional Components
```typescript
import React, { useState, useEffect } from 'react';

interface Props {
  title: string;
  onSubmit: (data: FormData) => void;
}

export const FormComponent: React.FC<Props> = ({ title, onSubmit }) => {
  const [value, setValue] = useState<string>('');
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({ value });
  };
  
  return (
    <form onSubmit={handleSubmit}>
      <h1>{title}</h1>
      <input 
        value={value}
        onChange={(e) => setValue(e.target.value)}
      />
    </form>
  );
};
```

### Custom Hooks
```typescript
function useApi<T>(url: string) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  
  useEffect(() => {
    fetch(url)
      .then(res => res.json())
      .then(setData)
      .catch(setError)
      .finally(() => setLoading(false));
  }, [url]);
  
  return { data, loading, error };
}
```

## Node.js Backend

### Express API
```typescript
import express, { Request, Response } from 'express';

interface CreateUserRequest {
  name: string;
  email: string;
}

interface UserResponse {
  id: string;
  name: string;
  email: string;
  createdAt: Date;
}

const app = express();
app.use(express.json());

app.post('/api/users', async (
  req: Request<{}, {}, CreateUserRequest>,
  res: Response<UserResponse | { error: string }>
) => {
  try {
    const user = await createUser(req.body);
    res.status(201).json(user);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});
```

### Validation with Zod
```typescript
import { z } from 'zod';

const UserSchema = z.object({
  name: z.string().min(1),
  email: z.string().email(),
  age: z.number().min(0).optional(),
});

type User = z.infer<typeof UserSchema>;

// Validate input
const result = UserSchema.safeParse(input);
if (!result.success) {
  console.error(result.error.issues);
}
```

## Testing

### Jest Tests
```typescript
import { describe, it, expect } from '@jest/globals';

describe('UserService', () => {
  it('should create a user', async () => {
    const service = new UserService();
    const user = await service.create({
      name: 'John',
      email: 'john@example.com'
    });
    
    expect(user.name).toBe('John');
    expect(user.email).toBe('john@example.com');
    expect(user.id).toBeDefined();
  });
  
  it('should throw on invalid email', async () => {
    const service = new UserService();
    await expect(
      service.create({ name: 'John', email: 'invalid' })
    ).rejects.toThrow('Invalid email');
  });
});
```

## Project Setup

### tsconfig.json Template
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "./dist",
    "rootDir": "./src"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### package.json Scripts
```json
{
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "test": "jest",
    "lint": "eslint src --ext .ts,.tsx",
    "typecheck": "tsc --noEmit"
  }
}
```

## Common Tasks

### Create New Component
1. Create file: `src/components/ComponentName.tsx`
2. Define Props interface
3. Write functional component
4. Add styles (CSS modules or styled-components)
5. Write tests: `ComponentName.test.tsx`

### Create API Endpoint
1. Define request/response types
2. Write handler function
3. Add validation
4. Implement business logic
5. Add error handling
6. Write tests

### Database Operations
```typescript
// With Prisma ORM
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

async function createUser(data: CreateUserInput) {
  return await prisma.user.create({
    data: {
      ...data,
      createdAt: new Date(),
    },
  });
}
```

## Validation Checklist

- [ ] All functions have explicit return types
- [ ] No `any` types without justification
- [ ] Error handling for all async operations
- [ ] Input validation with Zod or similar
- [ ] Tests cover happy path and error cases
- [ ] Code compiles without TypeScript errors
- [ ] ESLint passes with no warnings
