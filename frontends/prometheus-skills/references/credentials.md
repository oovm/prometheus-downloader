# credentials (on demand)

User-supplied cookies for hosts that need a **logged-in session**. This is a setup-skill job: you ask, they log in themselves, they hand you cookies. The product **does not** read cookies yet — credential APIs (header injection, vault storage) come later.

Use only cookies the user owns, and only for content they are allowed to fetch.

## When

Follow this guide when **any** of these is true. Skip it during a default CLI install if they never mentioned a session host.

- They named a host that often needs a login session (for example Bilibili), and a public API is not enough
- They asked to provide cookies, a cookie file, or a logged-in session
- `info` / `download` hit a **login wall** or an access challenge (for example HTTP 412), and they still want that host

Do **not** follow this guide to “make downloads more reliable”, to add Torch, or to install a headless browser.

## Ask first (required)

Skip a question only when this conversation already answered it. Ask **one or two** items per turn (AskQuestion when available). Stop until they answer.

### 1. Supply cookies at all?

- Yes — they will provide **their own** cookies for a host they can access
- No — leave the environment as-is; explain that this host may stay unavailable until they supply a session
- Not sure — explain that cookies are optional and only for hosts that need a login session, then ask again

### 2. Which host?

One host or registrable domain (example: `bilibili.com`). Do not guess a second site.

### 3. How will they collect cookies?

- **IDE built-in browser** (Cursor Simple Browser / the IDE’s browser tab): they open the site, log in themselves, then copy cookies or export a cookies.txt
- **Paste / file they already have**: they paste a `Cookie` header or a Netscape cookies.txt they exported from **their** browser
- **Their own browser, not the IDE**: same as paste — they log in, then hand you the string or file

Do **not** offer “read my Chrome/Edge/Firefox profile” as an option.

### 4. Where to keep them? (only after they agreed to supply)

Be honest: the **vault is a stub**. `createVault` writes an empty vault file; **encryption is not implemented**. Do not say cookies are “stored encrypted”.

- **Home file (default):** write Netscape cookies.txt to `~/.prometheus/cookies.txt` (Windows: `%USERPROFILE%\.prometheus\cookies.txt`), then set `PROMETHEUS_COOKIES_FILE` to that absolute path
- **Project file:** only if they asked to keep it in this workspace — add the path to **that project’s** `.gitignore` **before** writing, then write. Never `git add` it
- **Session env only:** set `PROMETHEUS_COOKIES` to a single-host `Cookie` header (`name=value; name2=value2`) for this terminal; do not persist
- **Wait for the vault:** do not write cookies anywhere; do not call `createVault` as a cookie store

The native CLI does **not** consume these values today. You are staging **user-supplied cookies** for a later credential API. Say that out loud.

## Steps (after they agree)

### A. Collect in the IDE browser

1. Open the IDE built-in browser (Simple Browser, or the IDE browser tab the user already uses).
2. Navigate to the host they named. They log in **themselves**. Do not type their password for them unless they paste it into a prompt they control; prefer they type in the browser.
3. After they are logged in, ask them to copy either:
   - the request `Cookie` header from that host’s DevTools Network panel, or
   - an export in **Netscape cookies.txt** shape (see below)
4. They paste into the chat, or save a file and give you the path.

If the IDE has no usable browser tab, ask them to use their own browser and paste. Do not install Playwright / Puppeteer / a headless browser to collect cookies.

### B. Collect by paste

They paste one of:

- A `Cookie` header value for that host
- A Netscape HTTP Cookie File

Accept only what they paste. Do not go looking on disk for other apps’ cookie databases.

### C. Stage (only the option they picked)

**Netscape cookies.txt** (home default):

```text
# Netscape HTTP Cookie File
.example.com	TRUE	/	FALSE	0	name	value
```

Fields are TAB-separated: `domain`, `flag`, `path`, `secure`, `expiration` (unix seconds, or `0`), `name`, `value`. Include a `# Netscape HTTP Cookie File` first line. Create the parent directory if needed (`~/.prometheus/`).

**Cookie header env** (single host):

```bash
# Unix
export PROMETHEUS_COOKIES='name=value; name2=value2'
export PROMETHEUS_COOKIES_FILE="$HOME/.prometheus/cookies.txt"

# Windows PowerShell
$env:PROMETHEUS_COOKIES = 'name=value; name2=value2'
$env:PROMETHEUS_COOKIES_FILE = "$env:USERPROFILE\.prometheus\cookies.txt"
```

After writing a file, set `PROMETHEUS_COOKIES_FILE` to its absolute path. Do not print the cookie values back in full in later messages; confirm the path / env name only.

Remind them: `info` / `download` still will not send these cookies until the credential API exists.

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| Agent read a browser profile folder | Violated this guide | Stop; delete nothing without asking; do not retry that approach |
| Agent installed Playwright to “get cookies” | Violated default path | Remove it if they did not ask for a real-browser session; apologize; return here |
| Agent called `createVault` and said cookies are encrypted | Vault is a stub | Correct that; keep cookies in the staging file/env they chose, or nowhere |
| Login wall / HTTP 412 on `info` | Host wants a session; engine does not inject cookies yet | Ask this guide’s questions; do not automate around the challenge |
| They wanted Bilibili (or similar) as `npm install` extra | Cookies ≠ extra plugins | Cookies: this guide. New inspect path: [plugin/references/explore.md](../plugin/references/explore.md) |

## Stop and ask

- They have not said whether to supply cookies
- Host is unclear
- Collection method is unclear (IDE browser vs paste)
- Persist location is unclear — especially anything that sounds like a “vault”
- They asked you to pull cookies from another app’s profile directory — **refuse**
- They asked you to automate around an access challenge — **refuse**; offer user-supplied cookies instead
