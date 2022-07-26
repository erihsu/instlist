module complex_logic
(
	input a1,
	input a2,
	output k
);

wire e,f;


middle_logic u_middle_logic(a1,a2,e,f);
low_logic u_low_logic2(a1,a2,e,f);
and_op u_and2(e,f,k);
and_op u_and3(e,f,k);

endmodule