using AtCoderHeuristicContest016.Sampler;
using Cysharp.Diagnostics;
using Newtonsoft.Json.Serialization;
using Newtonsoft.Json;
using System.Collections.Concurrent;
using System.Runtime.Intrinsics.X86;

var parameters = Enumerable.Range(4, 3).SelectMany(i => Enumerable.Range(0, 41).Select(j => (i, j))).Reverse();
await Task.WhenAll(BuildAsync("ahc016"), BuildAsync("sampler"));

var options = new ParallelOptions
{
    MaxDegreeOfParallelism = 32,
};

var directoryPath = @$"data\sampled\{DateTime.Now:yyyyMMdd_HHmmss}";
Directory.CreateDirectory(directoryPath);
var bag = new ConcurrentBag<Statistics>();

await Parallel.ForEachAsync(parameters, options, async (param, ct) =>
{
    var (bits, eps) = param;
    Console.WriteLine($"{bits} {eps}");
    var epsDouble = (double)eps / 100;

    var results = new List<QueryResult>();
    var output = ProcessX.StartAsync($"sampler.exe -b {bits} -e {epsDouble} -c ahc016.exe");
    await using var writer = new StreamWriter(Path.Combine(directoryPath, $"{bits}_{eps:00}.csv"));

    await foreach (var s in output)
    {
        writer.WriteLine(s);
        var pair = s.Split(',').Select(int.Parse).ToArray();
        results.Add(new QueryResult(pair[0], pair[1]));
    }

    var statistics = new Statistics(bits, epsDouble, results);
    bag.Add(statistics);
});

var contractResolver = new DefaultContractResolver
{
    NamingStrategy = new SnakeCaseNamingStrategy()
};

var json = JsonConvert.SerializeObject(bag.OrderBy(s => s.Bits).ThenBy(s => s.ErrorRatio).ToArray(), new JsonSerializerSettings
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