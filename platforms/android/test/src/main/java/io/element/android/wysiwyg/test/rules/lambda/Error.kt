package io.element.android.wysiwyg.test.rules.lambda

fun lambdaError(
    message: String = "This lambda should never be called."
): Nothing {
    throw AssertionError(message)
}
