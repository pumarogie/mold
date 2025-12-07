# mold

A blazingly fast CLI tool that generates **TypeScript types**, **Zod schemas**, and **Prisma models** from JSON.

## Installation

```bash
cargo install mold-cli
```

## Usage

```bash
# Generate TypeScript interfaces
mold schema.json --ts

# Generate Zod schema
mold schema.json --zod

# Generate Prisma model
mold schema.json --prisma

# Generate all formats
mold schema.json --all

# Output to files instead of stdout
mold schema.json --all -o ./generated

# Custom type name (default: inferred from filename)
mold data.json --ts --name User

# Flat mode - keep nested objects inline
mold schema.json --ts --flat
```

## Example

**Input (`user.json`):**
```json
{
  "id": 1,
  "name": "John Doe",
  "email": "john@example.com",
  "active": true,
  "profile": {
    "bio": "Developer",
    "avatar": "https://example.com/avatar.png"
  }
}
```

**Output (`--ts`):**
```typescript
interface UserProfile {
  avatar: string;
  bio: string;
}

interface User {
  active: boolean;
  email: string;
  id: number;
  name: string;
  profile: UserProfile;
}
```

**Output (`--zod`):**
```typescript
import { z } from "zod";

const UserProfileSchema = z.object({
  avatar: z.string(),
  bio: z.string(),
});

const UserSchema = z.object({
  active: z.boolean(),
  email: z.string(),
  id: z.number().int(),
  name: z.string(),
  profile: UserProfileSchema,
});

type UserProfile = z.infer<typeof UserProfileSchema>;
type User = z.infer<typeof UserSchema>;

export { UserProfileSchema, UserSchema };
export type { UserProfile, User };
```

**Output (`--prisma`):**
```prisma
model UserProfile {
  id     Int    @id @default(autoincrement())
  avatar String
  bio    String
}

model User {
  id     Int     @id @default(autoincrement())
  active Boolean
  email  String
  name   String
}
```

## Features

- **Type inference** - Automatically detects string, number, integer, boolean, null, arrays, and objects
- **Nested type extraction** - Nested objects are extracted as separate types/schemas
- **Union types** - Mixed arrays like `[1, "two", true]` become union types
- **Flat mode** - Keep nested objects inline with `--flat`
- **Multiple outputs** - Generate all formats at once with `--all`

## CLI Options

```
Usage: mold [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to JSON file

Options:
  -t, --ts            Generate TypeScript interfaces
  -z, --zod           Generate Zod schema
  -p, --prisma        Generate Prisma model
  -a, --all           Generate all formats
  -o, --output <DIR>  Output directory (default: stdout)
  -n, --name <NAME>   Root type name (default: inferred from filename)
      --flat          Keep nested objects inline (no extraction)
  -h, --help          Print help
  -V, --version       Print version
```

## License

MIT
