---
name: gcp-firebase-developer
description: "Build, configure, and operate Firebase-powered web and mobile applications — covering Firestore, Firebase Auth, Firebase Hosting, Cloud Functions for Firebase, Firebase Storage, App Check, Firebase Remote Config, and Firebase Analytics. Use when building mobile/web apps with Firebase, setting up authentication flows, designing Firestore data models, deploying Firebase Hosting, or configuring Firebase security rules."
allowed-tools: Read Grep Glob Bash(npm install:*) Bash(firebase:*)
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.1"
  updated: "2026-11-08"
  category: platform
---

# GCP Firebase Developer

## Overview

Firebase is Google's app development platform for web and mobile. It provides a unified suite of backend services, SDKs, and tooling. Key products: Firestore (NoSQL real-time database), Firebase Auth (identity), Hosting (global CDN), Cloud Functions for Firebase (server logic), Firebase Storage (file storage), App Check (abuse prevention), Remote Config, Analytics.

## Quick Start

```bash
# Install Firebase CLI
npm install -g firebase-tools

# Login
firebase login

# Initialize project
firebase init

# Deploy
firebase deploy
```

## Reference Directory

Load only when needed:

| Scenario | Trigger Keywords | Reference |
|---|---|---|
| Firestore data modeling | collection, document, subcollection, NoSQL schema, query design, index, transaction, TTL, PITR | [references/firestore.md](references/firestore.md) (claims cite [references/evidence-ledger.md](references/evidence-ledger.md)) |
| Firestore operations | backup, PITR, DR, replication, cost, billing, monitoring, IAM, tags, CMEK, VPC Service Controls, audit logs | Native mode: [references/firestore-operations.md](references/firestore-operations.md); MongoDB compatibility: [references/firestore-mongodb-operations.md](references/firestore-mongodb-operations.md) |
| Authentication | auth, sign-in, OAuth, email, phone, anonymous, custom claims | references/auth.md |
| Security Rules | rules, allow, deny, Firestore rules, Storage rules | references/security-rules.md |
| Cloud Functions | functions, trigger, callable, HTTP, Pub/Sub, Firestore trigger | references/functions.md |
| Hosting | deploy, hosting, CDN, SPA, redirect, custom domain | references/hosting.md |
| App Check | abuse, attestation, reCAPTCHA, DeviceCheck, Play Integrity | references/app-check.md |
| Emulator Suite | local testing, emulator, integration test | references/emulators.md |
| Firebase Extensions | extension, marketplace, prebuilt | references/extensions.md |

- [Audit](references/commands/audit.md) — offline or live audit against the operations reference; load only when an audit is requested.

## Core Operating Rules

- Firebase SDKs come in Web (v9 modular), React Native, Flutter, iOS (Swift), and Android (Kotlin/Java) variants. Always confirm the platform before providing SDK code.
- Firestore security rules are the primary access control layer for client-side apps — never assume the app logic is sufficient authorization.
- Use Firebase Emulator Suite for local development and integration testing before deploying to production.
- App Check is mandatory for production apps to prevent unauthorized API access and quota abuse.
- Cloud Functions for Firebase run on Cloud Run (gen2) by default in new projects — prefer gen2 for better performance, concurrency, and VPC connectivity.
- Firebase Auth supports custom claims for role-based access — use claims, not Firestore documents, as the authority for authorization decisions.
- Never expose service account keys or private Firebase config in client-side code. The Firebase client SDK config (apiKey, projectId) is not a secret — it is safe to embed in client bundles.
- Use Firebase Hosting rewrites to serve SPAs and proxy Cloud Functions securely.

## Response Shape

Platform/SDK confirmation, Firestore data model, security rules, auth flow, functions, hosting config, App Check setup, test plan with emulators.

## Official Docs

- https://firebase.google.com/docs
- https://firebase.google.com/docs/firestore
- https://firebase.google.com/docs/auth
- https://firebase.google.com/docs/hosting
- https://firebase.google.com/docs/functions
- https://firebase.google.com/docs/app-check

## Security Notes

Read-only skill. Do not deploy to production, modify Firestore security rules, or change Firebase project settings without explicit approval. Client config (apiKey, projectId) is public — service account keys are private and must never be embedded in client code.
