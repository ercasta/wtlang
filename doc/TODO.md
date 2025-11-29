# TODO

1. (Done) Expand the summary doc (summary.md) creating one file with additional design considerations for each topic (language, target platform, compiler and tools). These design file must contain an evaluation of possible alternatives, with rationale supporting to choice of the chosen one.
2. (Done) Added a consideration about testing: review the documentation (especially language design and support tools) to take into account testing
3. (Done) Create a language tutorial to explain the language, complete with typical usage examples. The tutorial must separately address the data presentation part (page creation) from data manipulation part (functions, chains, external calls).
4. (Done) Create a first implementation of the compiler. Generate some source files and test the compilation process by running the compiler to produce actual streamlit output pages. Keep all example source files and related output, they will form the basis for testing the compiler itself
5. (TODO) Start working on a first implementation of the language server to start experimenting with VSCode. The first step is updating the documentation regarding compiler tools, explaining the source code structure, that must support both the compiler and the Language Server (and going forward, more tools, such as debugger, documentation generator, etc.). Do not generate any implementation yet.
