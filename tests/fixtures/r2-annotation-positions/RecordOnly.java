import java.lang.annotation.*;
@Retention(RetentionPolicy.RUNTIME) @Target(ElementType.RECORD_COMPONENT)
@interface RecordMarker {}
record RecordOnly(@RecordMarker int value) {}
