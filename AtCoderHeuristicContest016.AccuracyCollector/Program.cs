using AtCoderHeuristicContest016.AccuracyCollector;
using Cysharp.Diagnostics;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;
using System.Collections.Concurrent;

var maxSizes = new int[] { -1, -1, -1, -1, 11, 34, 100 };
var mList = new int[] { 10, 11, 20, 25, 28, 30, 31, 32, 33, 34, 40, 50, 60, 70, 80, 90, 100 };
var scoreCoeffs = new double[] { 1.0, 1.5, 2.0, 2.5, 3.0 };

var parameters = Enumerable.Range(4, 3).SelectMany(i => mList.Select(j => (i, j))).SelectMany(p => scoreCoeffs.Select(c => (p.i, p.j, c)));
await Task.WhenAll(BuildAsync("ahc016"), BuildAsync("accuracy_collector"));

var options = new ParallelOptions
{
    MaxDegreeOfParallelism = 32,
};

var directoryPath = @$"data\accuracy\{DateTime.Now:yyyyMMdd_HHmmss}";
Directory.CreateDirectory(directoryPath);
var bag = new ConcurrentBag<Statistics>();

await Parallel.ForEachAsync(parameters, options, async (param, ct) =>
{
    var (bits, m, scoreCoef) = param;

    if (maxSizes[bits] < m)
    {
        return;
    }

    var minRedundancy = 1;

    for (int eps = 0; eps <= 40; eps++)
    {
        var epsDouble = (double)eps / 100;
        var bestRedundancy = minRedundancy;
        var bestExpected = 0.0;

        for (int redundancy = minRedundancy; redundancy * bits <= 100; redundancy++)
        {
            var graphSize = bits * redundancy;

            var output = ProcessX.StartAsync($"accuracy_collector.exe -b {bits} -e {epsDouble} -r {redundancy} -m {m} -s {scoreCoef} -c ahc016.exe");
            var list = new List<int>();

            await foreach (var s in output)
            {
                list.Add(int.Parse(s));
            }

            var trialCount = list[0];
            var accepted = list[1];
            var accuracy = (double)accepted / trialCount;
            const int actualTrialCount = 100;
            var expectedScore = ScoreCalcurator.CalculateExpectedScore(actualTrialCount, accuracy, graphSize);
            var statistics = new Statistics(m, epsDouble, bits, redundancy, scoreCoef, trialCount, accepted, accuracy, expectedScore);
            bag.Add(statistics);
            Console.WriteLine(statistics);

            if (bestExpected < expectedScore)
            {
                bestExpected = expectedScore;
                minRedundancy = redundancy;

                // 次に全問正解しても超えられない場合はスルー
                if (expectedScore > ScoreCalcurator.CalculateExpectedScore(actualTrialCount, 1.0, bits * (redundancy + 1)))
                {
                    break;
                }
            }
            else
            {
                break;
            }
        }
    }
});

var contractResolver = new DefaultContractResolver
{
    NamingStrategy = new SnakeCaseNamingStrategy()
};

var statistics = bag
    .OrderBy(s => s.Bits)
    .ThenBy(s => s.M)
    .ThenBy(s => s.ScoreCoef)
    .ThenBy(s => s.ErrorRatio)
    .ThenBy(s => s.Redundancy)
    .ToArray();

var json = JsonConvert.SerializeObject(statistics, new JsonSerializerSettings
{
    ContractResolver = contractResolver,
});

await File.WriteAllTextAsync(Path.Combine(directoryPath, "statistics.json"), json);

static async Task BuildAsync(string binName)
{
    var command = $"cargo build --release --bin {binName}";
    var (_, stdOut, stdError) = ProcessX.GetDualAsyncEnumerable(command);
    await Task.WhenAll(stdOut.WaitAsync(), stdError.WaitAsync());
    File.Move(@$"..\target\release\{binName}.exe", @$".\{binName}.exe");
}