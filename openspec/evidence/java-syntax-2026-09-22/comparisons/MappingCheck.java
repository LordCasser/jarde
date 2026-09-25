public class MappingCheck {
public static void main(String[] args) {
int cases=0;
{
float[] values = {Float.NaN, Float.NEGATIVE_INFINITY, -1.0f, -0.0f, 0.0f, 1.0f, Float.POSITIVE_INFINITY, Float.MIN_VALUE, Float.MAX_VALUE};
for (float a:values) for (float b:values) {
{
int result = a > b ? 1 : a == b ? 0 : a < b ? -1 : 1;
if ((result == 0) != (a == b)) throw new AssertionError("float:1:=="); cases++;
if ((result != 0) != (a != b)) throw new AssertionError("float:1:!="); cases++;
if ((result < 0) != (a < b)) throw new AssertionError("float:1:<"); cases++;
if ((result <= 0) != (a <= b)) throw new AssertionError("float:1:<="); cases++;
if ((result > 0) != (!(a <= b))) throw new AssertionError("float:1:>"); cases++;
if ((result >= 0) != (!(a < b))) throw new AssertionError("float:1:>="); cases++;
}
{
int result = a > b ? 1 : a == b ? 0 : a < b ? -1 : -1;
if ((result == 0) != (a == b)) throw new AssertionError("float:-1:=="); cases++;
if ((result != 0) != (a != b)) throw new AssertionError("float:-1:!="); cases++;
if ((result < 0) != (!(a >= b))) throw new AssertionError("float:-1:<"); cases++;
if ((result <= 0) != (!(a > b))) throw new AssertionError("float:-1:<="); cases++;
if ((result > 0) != (a > b)) throw new AssertionError("float:-1:>"); cases++;
if ((result >= 0) != (a >= b)) throw new AssertionError("float:-1:>="); cases++;
}
}
}
{
double[] values = {Double.NaN, Double.NEGATIVE_INFINITY, -1.0d, -0.0d, 0.0d, 1.0d, Double.POSITIVE_INFINITY, Double.MIN_VALUE, Double.MAX_VALUE};
for (double a:values) for (double b:values) {
{
int result = a > b ? 1 : a == b ? 0 : a < b ? -1 : 1;
if ((result == 0) != (a == b)) throw new AssertionError("double:1:=="); cases++;
if ((result != 0) != (a != b)) throw new AssertionError("double:1:!="); cases++;
if ((result < 0) != (a < b)) throw new AssertionError("double:1:<"); cases++;
if ((result <= 0) != (a <= b)) throw new AssertionError("double:1:<="); cases++;
if ((result > 0) != (!(a <= b))) throw new AssertionError("double:1:>"); cases++;
if ((result >= 0) != (!(a < b))) throw new AssertionError("double:1:>="); cases++;
}
{
int result = a > b ? 1 : a == b ? 0 : a < b ? -1 : -1;
if ((result == 0) != (a == b)) throw new AssertionError("double:-1:=="); cases++;
if ((result != 0) != (a != b)) throw new AssertionError("double:-1:!="); cases++;
if ((result < 0) != (!(a >= b))) throw new AssertionError("double:-1:<"); cases++;
if ((result <= 0) != (!(a > b))) throw new AssertionError("double:-1:<="); cases++;
if ((result > 0) != (a > b)) throw new AssertionError("double:-1:>"); cases++;
if ((result >= 0) != (a >= b)) throw new AssertionError("double:-1:>="); cases++;
}
}
}
System.out.println("mapping cases="+cases);
}
}
