package em25;

public final class Runner {
	private Runner() {
	}

	public static void main(String[] args) {
		System.out.println(BoxingAudit.boxInteger().getClass().getName() + ":" + BoxingAudit.boxInteger()
				+ ":" + (BoxingAudit.boxInteger() == BoxingAudit.boxInteger()));
		System.out.println("integer-bounds:" + BoxingAudit.integerMinimum() + ":"
				+ (BoxingAudit.integerMinimum() == BoxingAudit.integerMinimum()) + ":"
				+ BoxingAudit.integerMaximum() + ":"
				+ (BoxingAudit.integerMaximum() == BoxingAudit.integerMaximum()));
		System.out.println("integer-outside:" + BoxingAudit.integerBelowRange() + ":"
				+ (BoxingAudit.integerBelowRange() == BoxingAudit.integerBelowRange()) + ":"
				+ BoxingAudit.integerAboveRange() + ":"
				+ (BoxingAudit.integerAboveRange() == BoxingAudit.integerAboveRange()));
		System.out.println("integer-context:" + BoxingAudit.integerAsNumber().getClass().getName() + ":"
				+ BoxingAudit.integerConsumedByCall());
		System.out.println(BoxingAudit.boxBoolean().getClass().getName() + ":" + BoxingAudit.boxBoolean()
				+ ":" + (BoxingAudit.boxBoolean() == BoxingAudit.boxBoolean()));
		System.out.println("boolean-false:" + BoxingAudit.boxFalse().getClass().getName() + ":"
				+ BoxingAudit.boxFalse() + ":"
				+ (BoxingAudit.boxFalse() == BoxingAudit.boxFalse()));
		System.out.println(BoxingAudit.boxByte().getClass().getName() + ":" + BoxingAudit.boxByte()
				+ ":" + (BoxingAudit.boxByte() == BoxingAudit.boxByte()));
		System.out.println(BoxingAudit.boxShort().getClass().getName() + ":" + BoxingAudit.boxShort()
				+ ":" + (BoxingAudit.boxShort() == BoxingAudit.boxShort()));
		System.out.println(BoxingAudit.boxCharacter().getClass().getName() + ":" + BoxingAudit.boxCharacter()
				+ ":" + (BoxingAudit.boxCharacter() == BoxingAudit.boxCharacter()));
		System.out.println("character-bounds:" + (int) BoxingAudit.characterAsciiMaximum() + ":"
				+ (BoxingAudit.characterAsciiMaximum() == BoxingAudit.characterAsciiMaximum()) + ":"
				+ (int) BoxingAudit.characterAboveAscii() + ":"
				+ (BoxingAudit.characterAboveAscii() == BoxingAudit.characterAboveAscii()));
		System.out.println(BoxingAudit.boxLong().getClass().getName() + ":" + BoxingAudit.boxLong()
				+ ":" + (BoxingAudit.boxLong() == BoxingAudit.boxLong()));
		System.out.println(BoxingAudit.unboxOrDefault(null) + ":"
				+ BoxingAudit.unboxOrDefault(0L) + ":" + BoxingAudit.unboxOrDefault(7L));
		System.out.println(BoxingAudit.unbox(Boolean.TRUE) + ":" + BoxingAudit.unbox(Boolean.FALSE));
		try {
			BoxingAudit.unbox(null);
			throw new AssertionError("null Boolean should throw while unboxing");
		} catch (NullPointerException expected) {
			System.out.println("null-unbox:NPE");
		}
	}
}
