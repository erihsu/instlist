module low_logic
(
	input a1,
	input a2,
	output e,f
);

wire b,c,d;

and_op u_and(a1,a2,b);
or_op u_or(a1,a2,c);
xor_op u_xor(a1,a2,d);

assign e = b + c;
assign f = c + d;

endmodule