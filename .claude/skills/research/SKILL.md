---
name: research
description: "Use when the user wants a topic researched, docs or API facts gathered, or reading legwork delegated to a background agent."
---

<skill name="research" purpose="Research a Question and File the Findings" input="the question to research" user-input="$ARGUMENTS" output="the findings, filed as a research concept and listed in the canonical knowledge's index">

<goal persona="researcher">
You answer questions from primary sources and file the findings in the canonical knowledge. Research the question given in the input above and file the findings, following `schema-research`.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/research.md" when="always" />
<tool_call name="aokf_search" query="{existing findings on the topic}" when="always" />
</bootstrap_actions>

<process_actions>
<step name="SPIN UP A BACKGROUND AGENT" task="Spin up a background agent to do the research, so work continues while it reads. The remaining steps are its job" />
<step name="INVESTIGATE" task="Investigate the question against primary sources — official docs, source code, specs, first-party APIs — never a secondary write-up of them. Follow every claim back to the source that owns it" />
<step name="FILE THE FINDINGS" task="File the findings as a concept at `knowledge/research/research-{nnn}-{topic}.md` per `schema-research`" />
<step name="LIST IN THE INDEX" task="List the concept in the canonical knowledge's `index.md`" />
<gate check="knowledge validates to PASS per the core knowledge block" on-fail="fix every error" />
</process_actions>


<rules>
<rule level="SHALL">treat findings already in the canonical knowledge as input to the research</rule>
<rule level="SHALL">extend an existing concept on the topic rather than duplicating it</rule>
<rule level="SHALL">keep the citation the frontmatter's job, not the prose's</rule>
</rules>
</skill>
