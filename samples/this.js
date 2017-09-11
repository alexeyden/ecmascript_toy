var new_vec = function(x,y) {
    return {
	x: x,
	y: y,
	set: function(x,y) {
	    this.x = x;
	    this.y = y;
	    return this;
	},
	add: function(v) {
	    this.x = this.x + v.x;
	    this.y = this.y + v.y;
	    return this;
	},
	dot: function(v) {
	    return this.x * v.x + this.y * v.y;
	}
    };
};

var v1 = new_vec(0, 0).set(12, 4).add(new_vec(-2, -3));
var v2 = new_vec(0, 0).set(2, 0);

v1.add(v2);

var dot = v1.dot(v2);

std.io.println(v1.x, v1.y); // 12, 1
std.io.println(v2.x, v2.y); // 2, 0
std.io.println(dot); // 24

