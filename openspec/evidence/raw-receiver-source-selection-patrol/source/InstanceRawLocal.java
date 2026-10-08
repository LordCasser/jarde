public class InstanceRawLocal<T> { public T value; public void put(Object value) { InstanceRawLocal alias = this; alias.value = value; } }
