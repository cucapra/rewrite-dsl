# Language specification

## Lexical forms

```math
\begin{aligned}
\langle\text{letter}\rangle
  &\coloneqq \texttt{A–Z} \mid \texttt{a–z} \\
\langle\text{digit}\rangle
  &\coloneqq \texttt{0–9} \\
\langle\text{id}\rangle
  &\coloneqq (\texttt{\_} \mid \langle\text{letter}\rangle)^+
          (\texttt{\_} \mid \langle\text{letter}\rangle \mid \langle\text{digit}\rangle)^* \\
\langle\text{bitwidth}\rangle
  &\coloneqq \langle\text{digit}\rangle^+ \\
\langle\text{bin-digit}\rangle
  &\coloneqq \texttt{0} \mid \texttt{1} \\
\langle\text{oct-digit}\rangle
  &\coloneqq \texttt{0} \mid \texttt{1} \mid \texttt{2} \mid \texttt{3}
          \mid \texttt{4} \mid \texttt{5} \mid \texttt{6} \mid \texttt{7} \\
\langle\text{hex-digit}\rangle
  &\coloneqq \langle\text{digit}\rangle \mid \texttt{A–F} \mid \texttt{a–f} \\
\langle\text{bin-lit}\rangle
  &\coloneqq \texttt{0b}\,\langle\text{bin-digit}\rangle^+ \\
\langle\text{oct-lit}\rangle
  &\coloneqq \texttt{0o}\,\langle\text{oct-digit}\rangle^+ \\
\langle\text{hex-lit}\rangle
  &\coloneqq \texttt{0x}\,\langle\text{hex-digit}\rangle^+ \\
\langle\text{dec-lit}\rangle
  &\coloneqq \langle\text{digit}\rangle^+ \\
\langle\text{num-lit}\rangle
  &\coloneqq \langle\text{bin-lit}\rangle \mid \langle\text{oct-lit}\rangle
          \mid \langle\text{hex-lit}\rangle \mid \langle\text{dec-lit}\rangle
\end{aligned}
```

## Rewrite rules

```math
\begin{aligned}
\langle\text{file}\rangle
  &\coloneqq \langle\text{rw-rule}\rangle^* \\
\langle\text{rw-rule}\rangle
  &\coloneqq \texttt{rule}\;\langle\text{id}\rangle\;\langle\text{params}\rangle\;
     \texttt{\{}\;\langle\text{rule-body}\rangle\;\texttt{\}} \\
\langle\text{params}\rangle
  &\coloneqq \texttt{(}\;[\langle\text{id}\rangle
     (\texttt{,}\;\langle\text{id}\rangle)^*]\;\texttt{)} \\
\langle\text{rule-body}\rangle
  &\coloneqq \langle\text{bv-expr}\rangle\;\texttt{=>}\;\langle\text{bv-expr}\rangle
     \;[\langle\text{where}\rangle] \\
\langle\text{where}\rangle
  &\coloneqq \texttt{where}\;(\langle\text{assgn}\rangle\;\texttt{;})^+ \\
\langle\text{assgn}\rangle
  &\coloneqq \langle\text{id}\rangle\;\texttt{:=}\;\langle\text{w-expr}\rangle
\end{aligned}
```

## Bitvector expressions

```math
\begin{aligned}
\langle\text{bv-expr}\rangle
  &\coloneqq \langle\text{bv-atom}\rangle\;[\langle\text{w-suffix}\rangle] \\
\langle\text{w-suffix}\rangle
  &\coloneqq \texttt{:}\;\langle\text{w-atom}\rangle \\
\langle\text{bv-atom}\rangle
  &\coloneqq \langle\text{num-lit}\rangle
      \mid \langle\text{bv-cond}\rangle
      \mid \langle\text{fn-call}\rangle
      \mid \langle\text{id}\rangle
      \mid \langle\text{concat}\rangle \\
  &\quad\mid \texttt{(}\;\langle\text{un-expr}\rangle\;\texttt{)}
      \mid \texttt{(}\;\langle\text{bin-expr}\rangle\;\texttt{)}
      \mid \texttt{(}\;\langle\text{bv-cond}\rangle\;\texttt{)} \\
\langle\text{bv-cond}\rangle
  &\coloneqq \texttt{if}\;\langle\text{bv-expr}\rangle^\dag\;\texttt{then}\;
      \langle\text{bv-expr}\rangle\;\texttt{else}\;\langle\text{bv-expr}\rangle \\
\langle\text{un-expr}\rangle
  &\coloneqq \langle\text{un-op}\rangle\;\langle\text{bv-expr}\rangle \\
\langle\text{bin-expr}\rangle
  &\coloneqq \langle\text{bv-expr}\rangle\;\langle\text{bin-op}\rangle\;
      \langle\text{bv-expr}\rangle \\
\langle\text{un-op}\rangle
  &\coloneqq \texttt{!} \mid \texttt{-} \\
\langle\text{bin-op}\rangle
  &\coloneqq \texttt{+} \mid \texttt{-} \mid \texttt{*} \mid \texttt{\textasciicircum}
      \mid \texttt{<<} \mid \texttt{>>} \\
  &\quad\mid \texttt{>=} \mid \texttt{<=} \mid \texttt{>} \mid \texttt{<}
      \mid \texttt{\&} \mid \texttt{|} \\
\langle\text{fn-call}\rangle
  &\coloneqq \langle\text{id}\rangle\;\texttt{(}\;
      [\langle\text{bv-expr}\rangle(\texttt{,}\;\langle\text{bv-expr}\rangle)^*]
      \;\texttt{)} \\
\langle\text{concat}\rangle
  &\coloneqq \texttt{\{}\;\langle\text{bv-expr}\rangle\;\texttt{,}\;\langle\text{bv-expr}\rangle\;\texttt{\}}
\end{aligned}
```

$^\dag$​Assumed to have a width of one.

## Width expressions

```math
\begin{aligned}
\langle\text{w-expr}\rangle
  &\coloneqq \langle\text{w-cond}\rangle \mid \langle\text{w-binary}\rangle \\
\langle\text{w-cond}\rangle
  &\coloneqq \texttt{if}\;\langle\text{bool-expr}\rangle\;\texttt{then}\;
      \langle\text{w-expr}\rangle\;\texttt{else}\;\langle\text{w-expr}\rangle \\
\langle\text{w-binary}\rangle
  &\coloneqq \langle\text{w-atom}\rangle
      (\langle\text{w-add-op}\rangle\;\langle\text{w-atom}\rangle)^* \\
\langle\text{w-atom}\rangle
  &\coloneqq \langle\text{id}\rangle \mid \langle\text{bitwidth}\rangle
      \mid \texttt{(}\;\langle\text{w-expr}\rangle\;\texttt{)} \\
\langle\text{w-add-op}\rangle
  &\coloneqq \texttt{+} \mid \texttt{-}
\end{aligned}
```

## Boolean expressions

```math
\begin{aligned}
\langle\text{bool-expr}\rangle
  &\coloneqq \langle\text{bool-or}\rangle \\
\langle\text{bool-or}\rangle
  &\coloneqq \langle\text{bool-and}\rangle
      (\texttt{||}\;\langle\text{bool-and}\rangle)^* \\
\langle\text{bool-and}\rangle
  &\coloneqq \langle\text{bool-atom}\rangle
      (\texttt{\&\&}\;\langle\text{bool-atom}\rangle)^* \\
\langle\text{bool-atom}\rangle
  &\coloneqq \langle\text{bool-not}\rangle \mid \langle\text{bool-const}\rangle
      \mid \langle\text{w-comp}\rangle
      \mid \texttt{(}\;\langle\text{bool-expr}\rangle\;\texttt{)} \\
\langle\text{bool-not}\rangle
  &\coloneqq \texttt{!}\langle\text{bool-atom}\rangle \\
\langle\text{bool-const}\rangle
  &\coloneqq \texttt{true} \mid \texttt{false} \\
\langle\text{w-comp}\rangle
  &\coloneqq \langle\text{w-expr}\rangle\;\langle\text{comp-op}\rangle\;
      \langle\text{w-expr}\rangle \\
\langle\text{comp-op}\rangle
  &\coloneqq \texttt{>=} \mid \texttt{<=} \mid \texttt{==}
      \mid \texttt{>} \mid \texttt{<}
\end{aligned}
```
