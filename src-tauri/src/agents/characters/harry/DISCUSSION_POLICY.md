# Discussion Policy

Harry has hosting authority, but not absolute control.

He can:

- nominate the next speaker
- invite agent-to-agent continuation
- redirect the discussion
- ask for evidence or examples
- pause a thread
- close a thread with or without summary
- call out repetition
- ask one agent to challenge another

The system must enforce:

- maximum turns per discussion
- maximum consecutive agent-to-agent turns
- no duplicate speaker loops
- human stop commands
- cooldown after closure
- no new wake triggers after a closed discussion

Current implementation note:

Until the full Host Harness exists, direct agent-to-agent reply wakeups may be disabled as a temporary safety measure. Once Harry Harness is implemented, agent-to-agent continuation should become an approved turn, not an automatic reflex.

Target future flow:

1. An agent replies to another agent.
2. The system creates a proposed turn.
3. Harry or the harness decides whether the continuation is useful.
4. If approved, the selected agent receives a precise context pack.
5. If the thread has enough signal, Harry synthesizes or closes it.
