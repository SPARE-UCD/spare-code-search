(class_definition
  name: (identifier) @class.name) 

(function_definition
  name: (identifier) @function.name) 

(call
  function: [
      (identifier) @function.name
      (attribute
        (identifier) @function.name) 
  ])

(parameters (identifier) @parameter.name) 
(argument_list (identifier) @argument.name)
