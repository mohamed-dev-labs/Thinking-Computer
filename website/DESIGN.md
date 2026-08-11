# Thinking Computer Marketing Site Design

## Intent

The public site explains an open-source, VM-first CLI agent without pretending to be a hosted agent console. The design is inspired by the restraint and technical clarity associated with Vercel: near-black surfaces, high contrast, a deliberate type scale, thin geometric rules, generous whitespace, compact controls, and motion limited to useful emphasis.

## Visual system

| Element | Decision |
| --- | --- |
| Background | Near-black `#050505` with a subtle graphite dot-grid and radial vignette. |
| Text | Warm white primary text, muted gray supporting text, and a single cool-lilac status accent. |
| Typography | System sans for reading; a mono stack for command examples, technical markers, and status chips. |
| Geometry | One-pixel borders, small radii, exact column alignment, and square-like panels rather than soft cards. |
| Brand | Text-first `THINKING COMPUTER` wordmark treatment. No generated hero illustration is required. |
| Motion | Opacity/transform transitions only, under 240ms, disabled for reduced-motion users. |

## Information architecture

The page has a compact sticky top bar and five scroll sections: Hero, operating boundary, architecture, extension surface, and documentation/launch. The top bar links to the public repository, documentation, and a single `Get started` anchor. There is no traditional footer; the final launch section ends with one small open-source status line.

## Responsive rules

The primary layout uses a 12-column desktop grid that collapses to one column below 720px. Long command examples wrap safely. The navigation never requires hover, focus indicators are visible, semantic landmarks are used, and every external repository link is text-labeled.

## Content principles

Claims must reflect the repository. The site can say that Thinking Computer has a Rust policy core, local memory, optional Plugins, Skills, a TUI foundation, provider profiles, and policy-gated channels. It must not imply autonomous unrestricted computer control, cloud hosting, personal-device access, or a feature that does not exist in the public source.
