The goal is to generate as many queries candidates as possible, based on the recent edit in the code file. The query should include identifiers that as relevant as possible to the completion line. The identifiers can be functions, classes, navigation expression or other relevant terms.

The identifiers should be extracted based on the configuration provided in the `QueryGeneratorConfig`. The extraction strategy can be one of the following: `functions_and_classes`, `navigation_expression`, or `all_identifiers`.

1. Find all functions declaration and classes declarations in the diff, its prefix, and suffix. (use Tree sitter)
- Filter duplicated
- Rank them by their distance to the completion line, starting from the closest (in terms of node rows and columns). Keep their node position for later use.
- Filter to max number of terms (e.g., 6).
- The possible candiate queries for this strategy are:
    - function_a function_b class_a (naive, all function, class ranked by distance)
    - for functions classes that have arguments, join the whole definition, e.g., function_a.arg1.arg2 function_b.arg1 (regex-based query)
    - 1st_function/class_appeared.*last_function/class_appeared (regex-based query)
    - function_a or function_b or class_a (if `use_or_logic` is set to True, this will be the query candidate)
    - fist candidate but reduce the max number of terms to 5 (e.g., function_a function_b class_a)
    - fist candidate but reduce the max number of terms to 4 (e.g., function_a function_b)
    - first candidate but reduce the max number of terms to 3 (e.g., function_a)
- if no query candidates are found, use the `navigation_expression` strategy to extract identifiers.
2. Navigation expression strategy (Use tree sitter):
- Extract the navigation expression from the diff, its prefix, and suffix.
- Filter duplicated.
- Rank them by their distance to the completion line, starting from the closest (in terms of rows and columns).
- Filter to max number of terms (e.g., 6).
- The possible candidate queries for this strategy are:
    - navigation.expression.a navigation.expression.b (naive, all navigation expressions ranked by distance)
    - navigation expression a b (unpacked and remove duplicated identifiers inside the navigation expression, take max terms)
    - navigation.*b (regex-based query) (first identier appeared in the first navigation expression, last identifier appeared in the last navigation expression)
    - navigation or expression or a or b (unpacked but use or logic)
    - second candidate but reduce the max number of terms to 5 (e.g., navigation expression a)
    - second candidate but reduce the max number of terms to 4 (e.g., navigation expression)
    - second candidate but reduce the max number of terms to 3 (e.g., navigation)
- if no query candidates are found, use the `all_identifiers` strategy to extract identifiers.
3. All identifiers strategy:
- Extract all identifiers from the diff, its prefix, and suffix.
- Rank them by the number of occurrences in the diff.
- Filter to max number of terms (e.g., 6).
- The possible candidate queries for this strategy are:
    - identifier_a identifier_b (naive, all identifiers ranked by distance and occurrences)
    - identifier_a or identifier_b (if `use_or_logic` is set to True, this will be the query candidate)
    - identifier_a.*last_identifier (regex-based query) (first identifier appeared in the diff, last identifier appeared in the diff
    - first candidate but reduce the max number of terms to 5 (e.g., identifier_a identifier_b)
    - first candidate but reduce the max number of terms to 4 (e.g., identifier_a)
    - first candidate but reduce the max number of terms to 3 (e.g., identifier)