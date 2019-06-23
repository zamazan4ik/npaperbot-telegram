#include <CLI/CLI.hpp>

#include <tgbot/net/CurlHttpClient.h>
#include <tgbot/tgbot.h>

#include <nlohmann/json.hpp>

#include <boost/algorithm/string.hpp>

#include <chrono>
#include <cstdlib>
#include <iostream>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

std::mutex papersDatabase;

void updatePapersDatabase(nlohmann::json& papers, const std::string& papersDatabaseAddress)
{
    std::vector<TgBot::HttpReqArg> args;
    TgBot::CurlHttpClient httpClient;

    const TgBot::Url uri(papersDatabaseAddress);

    const std::string result = httpClient.makeRequest(uri, args);

    std::lock_guard<std::mutex> lockGuard(papersDatabase);
    papers = nlohmann::json::parse(result);
}

int main(int argc, char* argv[])
{
    CLI::App app("nPaperBot Telegram");

    std::string token;
    app.add_option("--token", token, "Telegram Bot API token")->required();

    int MaxResultCount = 20;
    app.add_option("--max-results-count", MaxResultCount, "Maximum results count per request");

    int MaxMessageLength = 2500;
    app.add_option("--max-message-length", MaxMessageLength, "Maximum result message length");

    std::string PapersDatabaseAddress = "https://raw.githubusercontent.com/wg21link/db/master/index.json";
    app.add_option("--database-address", PapersDatabaseAddress, "Online database address with papers");

    CLI11_PARSE(app, argc, argv);

    nlohmann::json papers;
    updatePapersDatabase(papers, PapersDatabaseAddress);

    std::thread updatePapersThread([&papers, &PapersDatabaseAddress]()
        {
            while(true)
            {
                using namespace std::chrono_literals;
                std::this_thread::sleep_for(std::chrono::duration(10min));

                updatePapersDatabase(papers, PapersDatabaseAddress);
            }
        });
    updatePapersThread.detach();

    TgBot::Bot bot(token);
    bot.getEvents().onCommand("paper", [&bot, &papers, MaxResultCount, MaxMessageLength](TgBot::Message::Ptr message)
        {
            std::string fixedMessage = message->text.substr();

            fixedMessage.erase(fixedMessage.begin(), fixedMessage.begin() + fixedMessage.find(' ') + 1);

            const std::string ResultFiller = "For the request \"" +  fixedMessage + "\":\n";
            std::string result = ResultFiller;
            bool anyResult = false;
            int resultCount = 0;

            std::lock_guard<std::mutex> lockGuard(papersDatabase);
            for(auto const& paperObject : papers.items())
            {
                const auto paper = paperObject.value();
                // If we cannot find any supported field - just skip this paper
                if(paper.find("type") == paper.end() || paper.find("title") == paper.end() ||
                    paper.find("author") == paper.end() || paper.find("link") == paper.end() ||
                    paper["type"].get<std::string>() != "paper")
                {
                    continue;
                }

                // Search by paper name, title and author
                const auto paperName = paperObject.key();
                const auto paperTitle = paper["title"].get<std::string>();
                const auto paperAuthor = paper["author"].get<std::string>();
                if(boost::algorithm::icontains(paperName, fixedMessage) ||
                   boost::algorithm::icontains(paperTitle, fixedMessage) ||
                   boost::algorithm::icontains(paperAuthor, fixedMessage))
                {
                    if(resultCount == MaxResultCount)
                    {
                        result += "There are more papers. Please use more precise query.";
                        break;
                    }

                    anyResult = true;
                    ++resultCount;
                    result += paper["title"].get<std::string>() + " from " +
                              paper["author"].get<std::string>() + "\n" + paper["link"].get<std::string>() + "\n\n";

                    if(result.size() > MaxMessageLength)
                    {
                        bot.getApi().sendMessage(message->chat->id, result);
                        result = ResultFiller;
                    }
                }
            }

            if(!anyResult)
            {
               result +=  "Found nothing. Sorry.";
            }

            if(result != ResultFiller)
            {
                bot.getApi().sendMessage(message->chat->id, result);
            }
        });

    bot.getEvents().onCommand("help", [&bot](TgBot::Message::Ptr message)
    {
        bot.getApi().sendMessage(message->chat->id, "Use \"/paper\" command with substring from a proposal title."
                                                    "Search works only for titles and authors. Search works as finding a substring in a string."
                                                    "Fuzzy search isn't supported yet.");
    });

    try
    {
        std::cout << "Bot username: " << bot.getApi().getMe()->username << std::endl;
        TgBot::TgLongPoll longPoll(bot);
        while (true)
        {
            std::cout << "Long poll started\n";
            longPoll.start();
        }
    }
    catch (const TgBot::TgException& e)
    {
        std::cout << "Telegram bot exception: " << e.what() << std::endl;
    }
    catch(const std::exception& e)
    {
        std::cout << "Exception: " << e.what() << std::endl;
    }
    return 0;
}