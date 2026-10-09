public class GenericNullObjectContextProbe { public <T extends Number> T value() { return null; } public Number value(Number x) { return x; } public Object caller() { return value(); } }
