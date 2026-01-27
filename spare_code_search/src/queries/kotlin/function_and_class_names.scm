
;;; Class definitions

;; Regular classes, data classes, sealed classes, abstract classes
(class_declaration
	(type_identifier) @class.name) 

;; Object declarations (singletons)
(object_declaration
    (type_identifier) @object.name) 

;; Companion objects
(class_body 
    (companion_object 
        (type_identifier)? @companion.name)  ; name is optional
    ) 


;; Interfaces
;;(interface_declaration
   ;; (type_identifier) @interface.name) @interface.declaration


;;; Function definitions

(function_declaration
	. (simple_identifier) @function.name) 

;;(getter
;;	("get")) @function.builtin.declaration
;;(setter
;;	("set")) @function.builtin.declaration

;;(primary_constructor) @constructor.declaration
;;(secondary_constructor
;;	("constructor")) @constructor.declaration

;(constructor_invocation
;	(user_type
;		(type_identifier) @constructor.name) ) 


;(parameter
;	(simple_identifier) @parameter.name)

;(parameter_with_optional_type
	;(simple_identifier) @parameter.name) 

; lambda parameters
;(lambda_literal
;	(lambda_parameters
;		(variable_declaration
;			(simple_identifier) @parameter.name)))

;;; Function calls

; function()
(call_expression
	. (simple_identifier) @function.name) 
; object.function() or object.property.function()
(call_expression
	(navigation_expression
		(navigation_suffix
			(simple_identifier) @function.name) . ))
