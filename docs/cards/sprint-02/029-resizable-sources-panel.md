- Sprint: 02
- Type: Feature
- Status: In review
- Depends on: 028 (PR #55)

# Card 029 - Resizable Sources panel

The user selected a resizable sidebar panel with its own header, search and Add
source control. Give Sources a proper framed desktop panel and a persistent
sidebar divider, preserving selection, playback, search and source removal.

## Acceptance criteria

- [x] Frame Sources with its own header/count, search, scrolling rows and footer.
- [x] Open and focus local/Dropbox source setup from Add source.
- [x] Resize by pointer capture and keyboard arrows/Home/End.
- [x] Double-click to reset and Escape to cancel a drag.
- [x] Remember width between launches and preserve it across temporary window narrowing.
- [x] Constrain the sidebar to 192-360px and retain at least 520px for main content.
- [x] Treat unavailable/corrupt saved preferences as typed failures and keep resizing usable.
- [x] Keep search, navigation, footer and artwork visible at 800 by 600.
- [x] Verify light/dark materials and 80-source scrolling in the browser.
- [ ] Complete ordered checks and two review cycles.

## Established patterns

Reviewed the MIT-licensed react-resizable-panels documented layout/accessibility
approach: https://github.com/bvaughn/react-resizable-panels.
Muzo has one fixed sidebar split, so native pointer capture and existing React
state suffice without a new layout dependency. No external source code is copied.

Follow the WAI-ARIA window-splitter pattern for labelled focusable separators,
values, controlled pane and keyboard movement:
https://www.w3.org/WAI/ARIA/apg/patterns/windowsplitter/
ARIA in HTML explicitly permits the separator role on a native button:
https://www.w3.org/TR/html-aria/
The focused ESLint allowed-role entry reflects that valid native widget while
retaining the handler/focus rules and the rest of the Airbnb baseline.

## Verification

Red-first tests cover the panel action, preference persistence, width bounds,
keyboard integration, pointer drag and double-click reset. Additional tests cover
Escape cancellation, viewport changes, invalid saved values and storage failure.
Browser verification used 80 sources: dragging 240 to 320px, keyboard maximum,
reset, min-window clamp to 280px with main content 520px, and restored 280px after
a new document. At 800 by 600, the list had 99px of visible height and 2560px of
content; sidebar scrollHeight equalled its 518px height and album art stayed
visible. Add source focused setup directly. Light and dark screenshots were
inspected. Source removal continues through the existing confirmation dialog.
