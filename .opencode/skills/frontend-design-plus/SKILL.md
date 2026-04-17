---
name: frontend-design-plus
description: Create distinctive, production-grade frontend interfaces with high design quality. Use this skill when the user asks to build web components, pages, artifacts, posters, or applications. Generates creative, polished code that avoids generic AI aesthetics. Enhanced with typography systems, theme-locking, and isolated dimension control.
metadata:
  short-description: "Advanced frontend design with typography systems + theme control."
license: Complete terms in LICENSE.txt
---

## Attribution & License

This skill is a derivative work combining the **Frontend Design Plugin** by **Prithvi Rajasekaran** and **Alexander Bricken** (Anthropic) with techniques from the **Prompting for Frontend Aesthetics** cookbook (Anthropic). See `LICENSE.txt` for the Apache 2.0 license terms.

---

This skill guides creation of distinctive, production-grade frontend interfaces that avoid generic "AI slop" aesthetics. Implement real working code with exceptional attention to aesthetic details, creative choices, and systematic design thinking.

The user provides frontend requirements: a component, page, application, or interface to build. They may include context about the purpose, audience, or technical constraints.

## Design Thinking

Before coding, understand the context and commit to a BOLD aesthetic direction:
- **Purpose**: What problem does this interface solve? Who uses it?
- **Tone**: Pick an extreme: brutally minimal, maximalist chaos, retro-futuristic, organic/natural, luxury/refined, playful/toy-like, editorial/magazine, brutalist/raw, art deco/geometric, soft/pastel, industrial/utilitarian, solarpunk, cyberpunk noir, Memphis design, Swiss modernist, etc. There are so many flavors to choose from. Use these for inspiration but design one that is true to the aesthetic direction.
- **Constraints**: Technical requirements (framework, performance, accessibility).
- **Differentiation**: What makes this UNFORGETTABLE? What's the one thing someone will remember?

**CRITICAL**: Choose a clear conceptual direction and execute it with precision. Bold maximalism and refined minimalism both work — the key is intentionality, not intensity.

Then implement working code (HTML/CSS/JS, React, Vue, etc.) that is:
- Production-grade and functional
- Visually striking and memorable
- Cohesive with a clear aesthetic point-of-view
- Meticulously refined in every detail

## Frontend Aesthetics Guidelines

### Typography System

Typography instantly signals quality. Choose fonts that are beautiful, unique, and interesting. Avoid generic fonts; opt instead for distinctive choices that elevate the interface.

**NEVER use:** Inter, Roboto, Open Sans, Lato, Arial, default system fonts.

**Font selection by context:**
- **Code / Developer aesthetic**: JetBrains Mono, Fira Code, Space Grotesk
- **Editorial / Publishing**: Playfair Display, Crimson Pro, Fraunces, Newsreader
- **Startup / Modern**: Clash Display, Satoshi, Cabinet Grotesk
- **Technical / Corporate**: IBM Plex family, Source Sans 3
- **Distinctive / Expressive**: Bricolage Grotesque, Obviously, General Sans, Syne

**Pairing principle**: High contrast = interesting. Pair a distinctive display font with a refined body font. Display + monospace, serif + geometric sans, or variable font across weights all create tension and sophistication.

**Use extremes in weight and scale**: 100/200 weight vs 800/900 — not 400 vs 600. Size jumps of 3x+ — not 1.5x. Pick one distinctive font, use it decisively. Load from Google Fonts. State your choice before coding.

NEVER converge on the same font across generations. Each design must select its own typographic identity.

### Color & Theme

Commit to a cohesive aesthetic. Use CSS variables for consistency. Dominant colors with sharp accents outperform timid, evenly-distributed palettes. Draw from IDE themes, cultural aesthetics, and art movements for inspiration — reference specific sources rather than inventing generic palettes.

### Motion & Interaction

Use animations for effects and micro-interactions. Prioritize CSS-only solutions for HTML. Use Motion library for React when available. Focus on high-impact moments: one well-orchestrated page load with staggered reveals (`animation-delay`) creates more delight than scattered micro-interactions. Use scroll-triggering and hover states that surprise.

### Spatial Composition

Unexpected layouts. Asymmetry. Overlap. Diagonal flow. Grid-breaking elements. Generous negative space OR controlled density.

### Backgrounds & Visual Details

Create atmosphere and depth rather than defaulting to solid colors. Layer CSS gradients, use geometric patterns, or add contextual effects that match the overall aesthetic. Apply creative forms like gradient meshes, noise textures, layered transparencies, dramatic shadows, decorative borders, custom cursors, and grain overlays.

## Anti-Patterns: The "AI Slop" Checklist

NEVER produce designs exhibiting these characteristics:
- Overused font families (Inter, Roboto, Arial, system fonts)
- Cliched color schemes (particularly purple gradients on white backgrounds)
- Predictable layouts and component patterns
- Cookie-cutter design that lacks context-specific character
- Converging on the same choices across generations (Space Grotesk, blue-purple palettes, etc.)

Interpret creatively and make unexpected choices that feel genuinely designed for the context. No design should be the same. Vary between light and dark themes, different fonts, different aesthetics.

## Advanced Technique: Isolated Dimension Control

When the user requests targeted refinement of a single design dimension, focus exclusively on that axis while preserving everything else. This produces faster, more predictable improvements.

For example, if the user says "make the typography better" — apply the typography system above without restructuring layout, color, or motion. If they say "add motion" — orchestrate animations without changing the visual identity.

## Advanced Technique: Theme Locking

When the user requests a specific aesthetic (e.g., "solarpunk", "brutalist", "art deco"), lock every design decision to that theme:
- Color palette derived from the aesthetic's visual language
- Typography that belongs to that era or movement
- Layout patterns authentic to the style
- Textures, patterns, and decorative elements that reinforce the theme
- Motion style appropriate to the aesthetic (e.g., mechanical for industrial, organic for nature-inspired)

**IMPORTANT**: Match implementation complexity to the aesthetic vision. Maximalist designs need elaborate code with extensive animations and effects. Minimalist or refined designs need restraint, precision, and careful attention to spacing, typography, and subtle details. Elegance comes from executing the vision well.

Remember: Claude is capable of extraordinary creative work. Don't hold back — show what can truly be created when thinking outside the box and committing fully to a distinctive vision.
