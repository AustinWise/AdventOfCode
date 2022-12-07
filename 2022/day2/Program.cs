
using System.Diagnostics;

int totalScore = 0;
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

    int moveDiff = myMove - opponentMove;

    Debug.Assert(moveDiff >= -2 && moveDiff <= 2);

    // normalize the distance
    if (moveDiff < 0)
        moveDiff += 3;

    int roundScore;
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

    totalScore += roundScore;
}

Console.WriteLine($"Day 2 part 1 answer: {totalScore}");
