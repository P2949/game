# Roadmaps

Use this index to tell current release work apart from historical architecture
milestones.

| Roadmap | Status | Notes |
| --- | --- | --- |
| [Architecture split](../architectural-improvement-roadmap.md) | Complete / historical | Engine, runtime, backend, and content crates are separated. |
| [Content authoring API](../content-authoring-api-roadmap.md) | Complete / historical | `game-kit` beginner and advanced authoring surfaces exist. |
| [Beginner authoring](../beginner-authoring-roadmap.md) | Complete / historical | Beginner builders, wrappers, events, scenes, assets, maps, and rules exist. |
| [Beginner productization](../beginner-productization-roadmap.md) | Complete / v0.2.0 productization | Tracks completed diagnostics, generated-project workflow, Tiled demo, and release gates. |
| [Content/engine boundary consolidation](content-engine-boundary-consolidation.md) | Current | Narrows root APIs, splits large modules, unifies map commands, improves diagnostics, and hardens CI/docs. |
| [Post-1.0 live data reload](post-1.0-live-data-reload.md) | Future / post-1.0 | Tracks full structural `assets/game.ron` reload without making it a 1.0 blocker. |
| [Post-1.0 API surface cleanup](post-1.0-api-surface-cleanup.md) | Superseded | Root export cleanup is handled by content/engine boundary consolidation. |
