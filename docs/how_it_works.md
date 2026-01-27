
# SpareCodeSearch - How to *search* for *code* context when you do not have a *spare* GPU

Short Answer: All you need is key-word search.

## Introduction
Code Language Models (CLMs) have shown great promise in generating code, given the right contexts in their input prompts. Retrieval-Augmented Generation (RAG) frameworks aim to enhance CLMs by including another module for retrieving relevant context to construct the input prompt.  However, these retrieval modules commonly use semantic search, calculating similarity among embedded vectors representation of text inputs. As a result, they often require substantial computational resources for training and hosting these embedded models, making them infeasible to integrate into lightweight applications such as in-IDE AI-based code completion. In this solution paper, we prove that using keyword-search is sufficient to retrieve relevant and useful code context inside large codebases, without the need for extensive GPU resources. The usefulness of code contexts found by our solution is demonstrated through their completion results on the Code Context Competition's benchmark, reaching 0.748 and 0.725 chRF scores on Kotlin and Python tracks, respectively.
## Technical Details
### System Architecture
This project aims to explore efficient methods for searching code without relying on extensive GPU resources. By leveraging keyword-based search techniques, we can provide a lightweight alternative for code context retrieval.
### Offline indexing phase with Zoekt
- Zoekt
- index into shards
### Online context retrieving phase
1. Zoekt Query Construction Strategies
2. Post-processing of search results


### Findings and Discussion


## Threats to Validity

## References