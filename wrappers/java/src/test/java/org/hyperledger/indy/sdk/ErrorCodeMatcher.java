package org.hyperledger.indy.sdk;

import org.hamcrest.Description;
import org.hamcrest.TypeSafeMatcher;

public class ErrorCodeMatcher extends TypeSafeMatcher<IndyException> {
	private ErrorCode expectedErrorCode;

	public ErrorCodeMatcher(ErrorCode errorCode) {
		this.expectedErrorCode = errorCode;
	}

	@Override
	protected boolean matchesSafely(IndyException e) {
		return expectedErrorCode.equals(e.getErrorCode());
	}

	@Override
	public void describeTo(Description description) {
		description.appendText("expect ").appendText(expectedErrorCode.name());
	}
}
