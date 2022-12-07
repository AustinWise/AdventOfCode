
using System.Diagnostics;

int partOneScore = 0;
int partTwoScore = 0;
foreach (var line in day2.Properties.Resources.Input.Split(new char[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries))
{
    Debug.Assert(line.Length == 3);
    Debug.Assert(line[1] == ' ');

    // A for Rock, B for Paper, and C for Scissors
    char opponentChar = line[0];
    // X for Rock, Y for Paper, and Z for Scissors
    char myChar = line[2];

    Debug.Assert(opponentChar >= 'A' && opponentChar <= 'C');
    Debug.Assert(myChar >= 'X' && myChar <= 'Z');

    // normalize to 0 for Rock, 1 for Paper, 2 for Scissors
    int opponentMove = opponentChar - 'A';
    int myMove = myChar - 'X';

    partOneScore += CalculateScore(opponentMove, myMove);

    myMove = myChar switch
    {
        // must loose
        'X' => ClampRange(opponentMove - 1),
        // must draw
        'Y' => opponentMove,
        // must win
        'Z' => ClampRange(opponentMove + 1),
        _ => throw new Exception(),
    };

    partTwoScore += CalculateScore(opponentMove, myMove);
}

Debug.Assert(partOneScore == 17189);
Debug.Assert(partTwoScore == 13490);

Console.WriteLine($"Day 2 part 1 answer: {partOneScore}");
Console.WriteLine($"Day 2 part 2 answer: {partTwoScore}");

// Keep in the range [0, 2) by adding or subtracting 3
static int ClampRange(int input)
{
    while (input < 0)
        input += 3;
    while (input > 2)
        input -= 3;
    return input;
}

static int CalculateScore(int opponentMove, int myMove)
{
    int roundScore;
    int moveDiff = myMove - opponentMove;

    Debug.Assert(moveDiff >= -2 && moveDiff <= 2);

    // normalize the distance
    moveDiff = ClampRange(moveDiff);

    if (moveDiff == 0)
    {
        // draw
        roundScore = 3;
    }
    else if (moveDiff == 1)
    {
        // win
        roundScore = 6;
    }
    else
    {
        Debug.Assert(moveDiff == 2);
        // loose
        roundScore = 0;
    }

    roundScore += (myMove + 1);
    return roundScore;
}