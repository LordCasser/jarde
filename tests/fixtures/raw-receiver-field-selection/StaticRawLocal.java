public class StaticRawLocal<T> { public T value; public static void put(StaticRawLocal receiver, Object value) { StaticRawLocal alias = receiver; alias.value = value; } }
