/** A record whose components exercise a name, a descriptor and a component annotation. */
public record RecordSample(int count, @Marker String label, long stamp) {}
