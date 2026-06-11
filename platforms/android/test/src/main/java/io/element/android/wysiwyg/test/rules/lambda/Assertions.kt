package io.element.android.wysiwyg.test.rules.lambda

fun assert(lambdaRecorder: LambdaRecorder): LambdaRecorderAssertions {
    return lambdaRecorder.assertions()
}

class LambdaRecorderAssertions internal constructor(
    private val parametersSequence: List<List<Any?>>,
) {
    fun isCalledOnce(): CalledOnceParametersAssertions {
        return CalledOnceParametersAssertions(
            assertions = isCalledExactly(1)
        )
    }

    fun isNeverCalled() {
        isCalledExactly(0)
    }

    fun isCalledExactly(times: Int): ParametersAssertions {
        if (parametersSequence.size != times) {
            throw AssertionError("Expected to be called $times, but was called ${parametersSequence.size} times")
        }
        return ParametersAssertions(parametersSequence)
    }
}

class CalledOnceParametersAssertions internal constructor(private val assertions: ParametersAssertions) {
    fun with(vararg matchers: ParameterMatcher) {
        assertions.withSequence(matchers.toList())
    }

    fun withNoParameter() {
        assertions.withNoParameter()
    }
}

class ParametersAssertions internal constructor(
    private val parametersSequence: List<List<Any?>>
) {
    fun withSequence(vararg matchersSequence: List<ParameterMatcher>) {
        if (parametersSequence.size != matchersSequence.size) {
            throw AssertionError("Lambda was called ${parametersSequence.size} times, but only ${matchersSequence.size} assertions were provided")
        }
        parametersSequence.zip(matchersSequence).forEachIndexed { invocationIndex, (parameters, matchers) ->
            if (parameters.size != matchers.size) {
                throw AssertionError("Expected ${matchers.size} parameters, but got ${parameters.size} parameters during invocation #$invocationIndex")
            }
            parameters.zip(matchers).forEachIndexed { paramIndex, (param, matcher) ->
                if (!matcher.match(param)) {
                    throw AssertionError(
                        "Parameter #$paramIndex does not match the expected value (actual=$param,expected=$matcher) during invocation #$invocationIndex"
                    )
                }
            }
        }
    }

    fun withNoParameter() {
        if (parametersSequence.any { it.isNotEmpty() }) {
            throw AssertionError("Expected no parameters, but got some")
        }
    }

    fun withFirstParameters(matchers: List<ParameterMatcher>) {
        withParametersAtCall(invocation = 0, matchers)
    }

    fun withParametersAtCall(invocation: Int, matchers: List<ParameterMatcher>) {
        if (parametersSequence.isEmpty()) {
            throw AssertionError("Expected to be called at least once, but was never called")
        }
        val parameters = parametersSequence[invocation]
        parameters.zip(matchers).forEachIndexed { paramIndex, (param, matcher) ->
            if (!matcher.match(param)) {
                throw AssertionError(
                    "Parameter #$paramIndex does not match the expected value (actual='$param', expected='$matcher') during invocation #$invocation"
                )
            }
        }
    }

    fun withLastParameters(matchers: List<ParameterMatcher>) {
        withParametersAtCall(invocation = parametersSequence.lastIndex, matchers)
    }
}
