* Sample for the ABAP pack: written for this repository, no foreign code.
REPORT zsample.

INTERFACE zif_shape PUBLIC.
  METHODS area RETURNING VALUE(rv_area) TYPE i.
  METHODS name RETURNING VALUE(rv_name) TYPE string.
ENDINTERFACE.

CLASS zcl_order DEFINITION PUBLIC CREATE PUBLIC.
  PUBLIC SECTION.
    INTERFACES zif_shape.
    METHODS constructor.
    METHODS calculate IMPORTING iv_weight TYPE i RETURNING VALUE(rv_fee) TYPE i.
  PRIVATE SECTION.
    METHODS helper.
ENDCLASS.

CLASS zcl_order IMPLEMENTATION.
  METHOD constructor.
    helper( ).
  ENDMETHOD.

  METHOD zif_shape~area.
    rv_area = 1.
  ENDMETHOD.

  METHOD calculate.
    " line comment
    IF iv_weight > 30 AND iv_weight < 90.
      rv_fee = 5.
    ELSEIF iv_weight = 0.
      rv_fee = 0.
    ELSE.
      LOOP AT mt_items INTO DATA(ls_item).
        PERFORM Add_Item USING ls_item.
        zcl_other=>RUN( ).
      ENDLOOP.
    ENDIF.
  ENDMETHOD.

  METHOD helper.
  ENDMETHOD.
ENDCLASS.

FORM add_item USING is_item TYPE ty_item.
  WRITE is_item-name.
ENDFORM.

FUNCTION z_send_document.
  CASE iv_kind.
    WHEN 1.
      rv = 1.
    WHEN OTHERS.
      rv = 2.
  ENDCASE.
ENDFUNCTION.
