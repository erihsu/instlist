module middle_logic
(
	input a1,
	input a2,
	output e,f
);

wire b,c,d;

and_op u_and(a1,a2,b);
or_op u_or(a1,a2,c);
xor_op u_xor(a1,a2,d);
low_logic u_low_logic3(a1,a2,e,f);

assign e = b + c;
assign f = c + d;

endmodule