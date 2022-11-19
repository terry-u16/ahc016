using MathNet.Numerics;

namespace AtCoderHeuristicContest016.AccuracyCollector;

internal class ScoreCalcurator
{
    public static double CalculateExpectedScore(int trialCount, double accuracy, int graphSize)
    {
        var expectedScore = 0.0;

        for (int acCount = 0; acCount <= trialCount; acCount++)
        {
            var errorCount = trialCount - acCount;
            var prob = CalcurateProbability(trialCount, acCount, accuracy);
            var score = 1e9 * Math.Pow(0.9, errorCount) / graphSize;
            expectedScore += score * prob;
        }

        return expectedScore;
    }

    private static double CalcurateProbability(int n, int k, double p)
    {
        return Combinatorics.Combinations(n, k) * Math.Pow(p, k) * Math.Pow(1 - p, n - k);
    }
}
