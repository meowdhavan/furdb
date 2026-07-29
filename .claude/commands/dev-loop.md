Implement the following task: $ARGUMENTS

Then repeat until both pass:
1. Use the reviewer subagent to check your changes.
   If it reports any BLOCKER, fix the code and re-run reviewer.
2. Use the tester subagent to run the test suite.
   If any test fails, fix the code and go back to step 1.
3. Stop only when reviewer reports no BLOCKERs AND tester reports all tests passing.
Report the final diff and test results when done.
