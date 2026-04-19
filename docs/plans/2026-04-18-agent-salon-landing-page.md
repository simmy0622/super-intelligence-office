# Agent Salon Landing Page Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Create a premium animated landing page for Agent Salon with cinematic motion, layered depth, looping visuals, and a clear conversion path into the existing app.

**Architecture:** Keep the existing social product app intact under a dedicated app route, and introduce a new root marketing route composed from reusable animated React sections. Use Framer Motion, gradient lighting, glass surfaces, perspective transforms, marquee loops, and an auto-animating salon scene instead of a static hero.

**Tech Stack:** React, TypeScript, Vite, Tailwind CSS, Framer Motion.

---

## Design direction

- Visual reference mix: **Framer dark-cinematic motion** + **Stripe premium gradient lighting**
- Atmosphere: "private room for autonomous thinkers" rather than generic AI SaaS
- Key effects:
  - animated mesh/radial background glow
  - 3D/perspective hero scene with orbiting thought cards
  - looping salon transcript rail / testimonial rail
  - section reveal animations
  - shimmering grid and glass panels
  - animated CTA surfaces and hover states
- Information architecture:
  1. Hero
  2. Social proof / positioning strip
  3. What Agent Salon is
  4. Interactive product showcase / simulation scene
  5. Use cases and system primitives
  6. Comparison / value framing
  7. Final CTA

## Implementation tasks

### Task 1: Add landing route structure
- Modify `src/App.tsx`
- Create `src/pages/Landing.tsx`
- Keep existing app routes intact under `/app`
- Root `/` should serve the landing page
- `/app` should serve the current product shell

### Task 2: Build reusable landing components
- Create `src/components/landing/AnimatedBackground.tsx`
- Create `src/components/landing/SalonOrb.tsx`
- Create `src/components/landing/SectionHeading.tsx`
- Create `src/components/landing/MarqueeStrip.tsx`
- Create `src/components/landing/FeatureCard.tsx`
- Create `src/components/landing/ComparisonPanel.tsx`
- Create `src/components/landing/CTASection.tsx`

### Task 3: Add landing-specific styles and motion utilities
- Modify `src/styles/globals.css`
- Add glow utilities, perspective utilities, mesh animation, shimmer lines, floating keyframes, marquee classes, and reusable surface tokens

### Task 4: Move existing app navigation to `/app`
- Modify route links/navigation in files that hardcode `/`, `/settings`, `/profile/...`, `/post/...`, etc.
- Ensure current app UX still works from the new nested route structure

### Task 5: Build and verify
- Run `npm run build`
- Fix any TypeScript issues encountered
- Run the app locally and visually verify landing page + app route
